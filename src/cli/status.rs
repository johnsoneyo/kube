use anyhow::Result;
use clap::Subcommand;

use crate::context::AppContext;
use crate::di::Executor;

#[derive(Subcommand, Debug)]
pub enum StatusCmd {
    /// Show full cluster overview
    Overview,

    /// Show only pod health
    Pods,

    /// Show only deployments health
    Deployments,

    /// Show only failing resources
    Issues,
}

pub fn handle(cmd: StatusCmd, exec: &dyn Executor, context: AppContext) -> Result<()> {
    match cmd {
        StatusCmd::Overview => show_overview(exec, context),
        StatusCmd::Pods => show_pods(exec, context),
        StatusCmd::Deployments => show_deployments(exec, context),
        StatusCmd::Issues => show_issues(exec, context),
    }
}

pub fn show_overview(exec: &dyn Executor, context: AppContext) -> Result<()> {
    let pod_output = run_kubectl(exec, &context, &["get", "pods", "-A"])?;
    let deployment_output = run_kubectl(exec, &context, &["get", "deployments", "-A"])?;

    let pod_summary = summarize_pods(&pod_output)?;
    let deployment_summary = summarize_deployments(&deployment_output)?;

    println!("Cluster overview");
    println!(
        "Pods: total={}, healthy={}, warnings={}, failing={}",
        pod_summary.total, pod_summary.healthy, pod_summary.warning, pod_summary.failing
    );
    println!(
        "Deployments: total={}, ready={}, unavailable={}",
        deployment_summary.total, deployment_summary.ready, deployment_summary.unavailable
    );
    Ok(())
}

pub fn show_pods(exec: &dyn Executor, context: AppContext) -> Result<()> {
    let output = run_kubectl(exec, &context, &["get", "pods", "-A"])?;
    let summary = summarize_pods(&output)?;
    println!(
        "Pods: total={}, healthy={}, warnings={}, failing={}",
        summary.total, summary.healthy, summary.warning, summary.failing
    );
    Ok(())
}

pub fn show_deployments(exec: &dyn Executor, context: AppContext) -> Result<()> {
    let output = run_kubectl(exec, &context, &["get", "deployments", "-A"])?;
    let summary = summarize_deployments(&output)?;
    println!(
        "Deployments: total={}, ready={}, unavailable={}",
        summary.total, summary.ready, summary.unavailable
    );
    Ok(())
}

pub fn show_issues(exec: &dyn Executor, context: AppContext) -> Result<()> {
    let output = run_kubectl(exec, &context, &["get", "pods", "-A"])?;
    let mut matches = Vec::new();

    for line in output.lines().skip(1) {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let lower = trimmed.to_ascii_lowercase();
        if lower.contains("crashloopbackoff")
            || lower.contains("imagepullbackoff")
            || lower.contains("pending")
            || lower.contains("error")
            || lower.contains("invalidimage")
            || lower.contains("unknown")
        {
            matches.push(trimmed.to_string());
        }
    }

    if matches.is_empty() {
        println!("No failing resources detected.");
        return Ok(());
    }

    for entry in matches {
        println!("{entry}");
    }
    Ok(())
}

fn run_kubectl(exec: &dyn Executor, context: &AppContext, tail: &[&str]) -> Result<String> {
    let mut args = context.kubectl_base_args();
    args.extend(tail.iter().map(|item| (*item).to_string()));
    let args_ref: Vec<&str> = args.iter().map(String::as_str).collect();
    exec.run(&args_ref)
}

#[derive(Debug, Default, PartialEq, Eq)]
pub struct PodStatusSummary {
    pub total: usize,
    pub healthy: usize,
    pub warning: usize,
    pub failing: usize,
}

#[derive(Debug, Default, PartialEq, Eq)]
pub struct DeploymentStatusSummary {
    pub total: usize,
    pub ready: usize,
    pub unavailable: usize,
}

pub fn summarize_pods(output: &str) -> Result<PodStatusSummary> {
    let mut summary = PodStatusSummary::default();
    for line in output.lines().skip(1) {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        summary.total += 1;
        let lower = trimmed.to_ascii_lowercase();
        if lower.contains("running") || lower.contains("completed") {
            summary.healthy += 1;
        } else if lower.contains("pending")
            || lower.contains("crashloopbackoff")
            || lower.contains("imagepullbackoff")
            || lower.contains("createcontainerconfigerror")
        {
            summary.warning += 1;
        } else {
            summary.failing += 1;
        }
    }
    Ok(summary)
}

pub fn summarize_deployments(output: &str) -> Result<DeploymentStatusSummary> {
    let mut summary = DeploymentStatusSummary::default();
    for line in output.lines().skip(1) {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        summary.total += 1;
        let parts: Vec<&str> = trimmed.split_whitespace().collect();
        if parts.len() < 3 {
            continue;
        }
        let ready = parts[1];
        if let Some((current, total)) = ready.split_once('/') {
            if current == total {
                summary.ready += 1;
            } else {
                summary.unavailable += 1;
            }
        } else {
            summary.unavailable += 1;
        }
    }
    Ok(summary)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Debug)]
    struct MockExecutor {
        output: String,
        captured_args: std::sync::Arc<std::sync::Mutex<Vec<Vec<String>>>>,
    }

    impl MockExecutor {
        fn new(output: &str) -> Self {
            Self {
                output: output.to_string(),
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
            Ok(self.output.clone())
        }

        fn stream(&self, _args: &[&str], _on_line: &mut dyn FnMut(&str)) -> anyhow::Result<()> {
            Ok(())
        }
    }

    #[test]
    fn show_pods_includes_context_and_namespace() {
        let exec = MockExecutor::new("NAME\tSTATUS\napi\tRunning\n");
        let context = AppContext {
            namespace: Some("payments".to_string()),
            kube_context: Some("prod".to_string()),
        };

        let result = show_pods(&exec, context);
        assert!(result.is_ok());
        assert_eq!(
            exec.args_for_last_call(),
            vec!["--context", "prod", "-n", "payments", "get", "pods", "-A"]
        );
    }

    #[test]
    fn summarize_pods_counts_healthy_and_failing_resources() {
        let output = "NAME\tREADY\tSTATUS\tRESTARTS\napi-1\t1/1\tRunning\t0\nweb\t0/1\tCrashLoopBackOff\t5\nworker\t0/1\tPending\t0\n";
        let summary = summarize_pods(output).expect("pod summary should parse");
        assert_eq!(summary.total, 3);
        assert_eq!(summary.healthy, 1);
        assert_eq!(summary.warning, 2);
        assert_eq!(summary.failing, 0);
    }

    #[test]
    fn summarize_deployments_counts_ready_and_unavailable_replicas() {
        let output =
            "NAME\tREADY\tUP-TO-DATE\tAVAILABLE\tAGE\napi\t2/2\t2\t2\t1d\nweb\t1/2\t2\t1\t2h\n";
        let summary = summarize_deployments(output).expect("deployments summary should parse");
        assert_eq!(summary.total, 2);
        assert_eq!(summary.ready, 1);
        assert_eq!(summary.unavailable, 1);
    }
}
