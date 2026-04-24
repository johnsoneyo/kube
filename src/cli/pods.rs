use clap::Subcommand;

use crate::context::AppContext;
use crate::di::{Executor};

#[derive(Subcommand, Debug)]
pub enum PodsCmd {
    List {
        filter: Option<String>
    }
}

pub fn handle(cmd: PodsCmd, exec: &dyn Executor, _context: AppContext) {

    match cmd {
        PodsCmd::List { filter } => {
            list_pods(exec, filter.as_deref(), _context);
        }
    }
}


pub fn list_pods(exec: &dyn Executor, filter: Option<&str>, context: AppContext) {
    let mut args = context.kubectl_base_args();

    args.push("get".to_string());
    args.push("pods".to_string());

    let output = exec.run(&args.iter().map(|s| s.as_str()).collect::<Vec<&str>>());

    for line in output.lines() {
        if line.starts_with("NAME") {
            continue;
        }

        match filter {
            Some(f) if !f.is_empty() => {
                if line.contains(f) {
                    println!("{line}");
                }
            }
            _ => println!("{line}"),
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    struct MockExecutor {
        pub output: String,
        pub captured_args: Vec<Vec<String>>,
    }

    impl MockExecutor {
        fn new(output: &str) -> Self {
            Self {
                output: output.to_string(),
                captured_args: vec![],
            }
        }
    }

    impl Executor for MockExecutor {
        fn run(&self, _args: &[&str]) -> String {
            // NOTE: we can't mutate self here because &self
            // so we only return output for now
            self.output.clone()
        }

        fn stream(&self, args: &[&str], on_line: &mut dyn FnMut(&str)) {
            todo!()
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
        let exec = MockExecutor::new(
            "NAME\nnginx-123\napi-456"
        );

        list_pods(&exec, None, empty_context());
    }

    #[test]
    fn list_pods_with_filter() {
        let exec = MockExecutor::new(
            "NAME\nnginx-123\napi-456"
        );

        list_pods(&exec, Some("nginx"), empty_context());
    }
}