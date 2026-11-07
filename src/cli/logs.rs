use anyhow::Result;
use clap::Subcommand;

use crate::context::AppContext;
use crate::di::Executor;

#[derive(Subcommand, Debug)]
pub enum LogsCmd {
    Follow { name: String, tail: Option<usize> },
    Show { name: String, tail: Option<usize> },
}

pub fn handle(cmd: LogsCmd, exec: &dyn Executor, context: AppContext) -> Result<()> {
    match cmd {
        LogsCmd::Follow { name, tail } => show_logs(exec, name, tail, context, true),
        LogsCmd::Show { name, tail } => show_logs(exec, name, tail, context, false),
    }
}

pub fn show_logs(
    exec: &dyn Executor,
    name: String,
    tail: Option<usize>,
    app_context: AppContext,
    follow: bool,
) -> Result<()> {
    let mut args = app_context.kubectl_base_args();

    args.push("logs".to_string());
    if follow {
        args.push("-f".to_string());
    }
    if let Some(count) = tail {
        args.push("--tail".to_string());
        args.push(count.to_string());
    }
    args.push(name);

    let args_ref: Vec<&str> = args.iter().map(String::as_str).collect();
    exec.stream(&args_ref, &mut |line| {
        println!("{line}");
    })?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Debug)]
    struct MockExecutor {
        captured_args: std::sync::Arc<std::sync::Mutex<Vec<Vec<String>>>>,
    }

    impl MockExecutor {
        fn new() -> Self {
            Self {
                captured_args: std::sync::Arc::new(std::sync::Mutex::new(vec![])),
            }
        }

        fn args_for_last_call(&self) -> Vec<String> {
            self.captured_args
                .lock()
                .expect("mutex poisoned")
                .last()
                .cloned()
                .unwrap_or_default()
        }
    }

    impl Executor for MockExecutor {
        fn run(&self, args: &[&str]) -> anyhow::Result<String> {
            self.captured_args
                .lock()
                .expect("mutex poisoned")
                .push(args.iter().map(|s| (*s).to_string()).collect());
            Ok("".to_string())
        }

        fn stream(&self, args: &[&str], on_line: &mut dyn FnMut(&str)) -> anyhow::Result<()> {
            self.captured_args
                .lock()
                .expect("mutex poisoned")
                .push(args.iter().map(|s| (*s).to_string()).collect());
            on_line("line-1");
            on_line("line-2");
            Ok(())
        }
    }

    #[test]
    fn show_logs_streams_context_aware_output() {
        let exec = MockExecutor::new();
        let context = AppContext {
            namespace: Some("default".to_string()),
            kube_context: Some("dev".to_string()),
        };

        let result = show_logs(&exec, "api".to_string(), Some(25), context, true);
        assert!(result.is_ok());
        assert_eq!(
            exec.args_for_last_call(),
            vec![
                "--context",
                "dev",
                "-n",
                "default",
                "logs",
                "-f",
                "--tail",
                "25",
                "api"
            ]
        );
    }
}
