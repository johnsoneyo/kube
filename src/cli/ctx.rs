use anyhow::Result;
use clap::Subcommand;

use crate::context::AppContext;
use crate::di::Executor;

#[derive(Subcommand, Debug)]
pub enum CtxCmd {
    Current,
    Use { name: String },
    List,
}

pub fn handle(cmd: CtxCmd, exec: &dyn Executor, _context: AppContext) -> Result<()> {
    match cmd {
        CtxCmd::Current => current_context(exec),
        CtxCmd::Use { name } => use_context(exec, name),
        CtxCmd::List => list_contexts(exec),
    }
}

pub fn current_context(exec: &dyn Executor) -> Result<()> {
    let output = exec.run(&["config", "current-context"])?;
    println!("{}", output.trim());
    Ok(())
}

pub fn use_context(exec: &dyn Executor, name: String) -> Result<()> {
    let output = exec.run(&["config", "use-context", &name])?;
    println!("{}", output.trim());
    Ok(())
}

pub fn list_contexts(exec: &dyn Executor) -> Result<()> {
    let output = exec.run(&["config", "get-contexts", "-o", "name"])?;
    for line in output.lines() {
        if !line.trim().is_empty() {
            println!("{line}");
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockExecutor {
        output: String,
    }

    impl MockExecutor {
        fn new(output: &str) -> Self {
            Self {
                output: output.to_string(),
            }
        }
    }

    impl Executor for MockExecutor {
        fn run(&self, _args: &[&str]) -> anyhow::Result<String> {
            Ok(self.output.clone())
        }

        fn stream(&self, _args: &[&str], _on_line: &mut dyn FnMut(&str)) -> anyhow::Result<()> {
            Ok(())
        }
    }

    #[test]
    fn current_context_prints_a_context_name() {
        let exec = MockExecutor::new("minikube\n");
        let result = current_context(&exec);
        assert!(result.is_ok());
    }

    #[test]
    fn use_context_accepts_context_name() {
        let exec = MockExecutor::new("Switched to context \"prod\".\n");
        let result = use_context(&exec, "prod".to_string());
        assert!(result.is_ok());
    }
}
