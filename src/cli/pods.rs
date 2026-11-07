use anyhow::Result;
use clap::Subcommand;

use crate::context::AppContext;
use crate::di::Executor;

#[derive(Subcommand, Debug)]
pub enum PodsCmd {
    List { filter: Option<String> },
}

pub fn handle(cmd: PodsCmd, exec: &dyn Executor, context: AppContext) -> Result<()> {
    match cmd {
        PodsCmd::List { filter } => list_pods(exec, filter.as_deref(), context),
    }
}

pub fn list_pods(exec: &dyn Executor, filter: Option<&str>, context: AppContext) -> Result<()> {
    let mut args = context.kubectl_base_args();

    args.push("get".to_string());
    args.push("pods".to_string());

    let output = exec.run(&args.iter().map(String::as_str).collect::<Vec<_>>())?;

    for line in output.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with("NAME") {
            continue;
        }

        match filter {
            Some(f) if !f.is_empty() => {
                if trimmed.contains(f) {
                    println!("{trimmed}");
                }
            }
            _ => println!("{trimmed}"),
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Debug)]
    struct MockExecutor {
        output: String,
        error: Option<String>,
        captured_args: std::sync::Arc<std::sync::Mutex<Vec<Vec<String>>>>,
    }

    impl MockExecutor {
        fn new(output: &str) -> Self {
            Self {
                output: output.to_string(),
                error: None,
                captured_args: std::sync::Arc::new(std::sync::Mutex::new(vec![])),
            }
        }

        fn with_error(output: &str, message: &str) -> Self {
            Self {
                output: output.to_string(),
                error: Some(message.to_string()),
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
        fn run(&self, args: &[&str]) -> Result<String, anyhow::Error> {
            self.captured_args
                .lock()
                .expect("mutex poisoned")
                .push(args.iter().map(|s| (*s).to_string()).collect());
            if let Some(ref error) = self.error {
                return Err(anyhow::anyhow!(error.clone()));
            }
            Ok(self.output.clone())
        }

        fn stream(
            &self,
            _args: &[&str],
            _on_line: &mut dyn FnMut(&str),
        ) -> Result<(), anyhow::Error> {
            Ok(())
        }
    }

    fn empty_context() -> AppContext {
        AppContext {
            namespace: None,
            kube_context: None,
        }
    }

    #[test]
    fn list_pods_runs_without_filter() {
        let exec = MockExecutor::new("NAME\nnginx-123\napi-456");

        let result = list_pods(&exec, None, empty_context());
        assert!(result.is_ok());
        assert_eq!(exec.args_for_last_call(), vec!["get", "pods"]);
    }

    #[test]
    fn list_pods_with_filter() {
        let exec = MockExecutor::new("NAME\nnginx-123\napi-456");

        let result = list_pods(&exec, Some("nginx"), empty_context());
        assert!(result.is_ok());
        assert_eq!(exec.args_for_last_call(), vec!["get", "pods"]);
    }

    #[test]
    fn list_pods_includes_context_and_namespace() {
        let exec = MockExecutor::new("NAME\nnginx-123");
        let context = AppContext {
            namespace: Some("payments".to_string()),
            kube_context: Some("dev-cluster".to_string()),
        };

        let result = list_pods(&exec, None, context);
        assert!(result.is_ok());
        assert_eq!(
            exec.args_for_last_call(),
            vec!["--context", "dev-cluster", "-n", "payments", "get", "pods"]
        );
    }

    #[test]
    fn list_pods_returns_error_when_executor_fails() {
        let exec = MockExecutor::with_error("NAME\n", "kubectl unavailable");
        let result = list_pods(&exec, None, empty_context());
        assert!(result.is_err());
    }
}
