use anyhow::{anyhow, Result};
use clap::Subcommand;
use serde::Deserialize;

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
    let pod_output = run_kubectl_json(exec, &context, "pods")?;
    let deployment_output = run_kubectl_json(exec, &context, "deployments")?;

    let pod_summary = summarize_pods(&pod_output)?;
    let deployment_summary = summarize_deployments(&deployment_output)?;

    println!("Cluster overview");
    println!(
        "Pods: total={}, healthy={}, warnings={}, failing={}",
        pod_summary.total, pod_summary.healthy, pod_summary.warning, pod_summary.failing
    );
    println!(
        "Deployments: total={}, ready={}, partial={}, unavailable={}",
        deployment_summary.total,
        deployment_summary.ready,
        deployment_summary.partial,
        deployment_summary.unavailable
    );
    Ok(())
}

pub fn show_pods(exec: &dyn Executor, context: AppContext) -> Result<()> {
    let output = run_kubectl_json(exec, &context, "pods")?;
    let summary = summarize_pods(&output)?;
    println!(
        "Pods: total={}, healthy={}, warnings={}, failing={}",
        summary.total, summary.healthy, summary.warning, summary.failing
    );
    Ok(())
}

pub fn show_deployments(exec: &dyn Executor, context: AppContext) -> Result<()> {
    let output = run_kubectl_json(exec, &context, "deployments")?;
    let summary = summarize_deployments(&output)?;
    println!(
        "Deployments: total={}, ready={}, partial={}, unavailable={}",
        summary.total, summary.ready, summary.partial, summary.unavailable
    );
    Ok(())
}

pub fn show_issues(exec: &dyn Executor, context: AppContext) -> Result<()> {
    let output = run_kubectl_json(exec, &context, "pods")?;
    let issues = pod_issues(&output)?;

    if issues.is_empty() {
        println!("No failing resources detected.");
        return Ok(());
    }

    for entry in issues {
        println!("{entry}");
    }
    Ok(())
}

fn run_kubectl_json(exec: &dyn Executor, context: &AppContext, resource: &str) -> Result<String> {
    let args = context.kubectl_status_args(resource);
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
    pub partial: usize,
    pub unavailable: usize,
}

#[derive(Debug, Deserialize, Default)]
struct KubernetesList<T> {
    #[serde(default)]
    items: Vec<T>,
}

#[derive(Debug, Deserialize, Default)]
struct PodResource {
    #[serde(default)]
    metadata: Metadata,
    #[serde(default)]
    status: PodStatus,
}

#[derive(Debug, Deserialize, Default)]
struct Metadata {
    #[serde(default)]
    name: String,
}

#[derive(Debug, Deserialize, Default)]
struct PodStatus {
    #[serde(default)]
    phase: String,
    #[serde(default)]
    conditions: Vec<PodCondition>,
    #[serde(default, rename = "containerStatuses")]
    container_statuses: Vec<ContainerStatus>,
    #[serde(default, rename = "initContainerStatuses")]
    init_container_statuses: Vec<ContainerStatus>,
}

#[derive(Debug, Deserialize, Default)]
struct PodCondition {
    #[serde(rename = "type")]
    type_: String,
    #[serde(default)]
    status: String,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize, Default)]
struct ContainerStatus {
    #[serde(default)]
    name: String,
    #[serde(default)]
    ready: bool,
    #[serde(default)]
    state: Option<ContainerState>,
    #[serde(default)]
    last_state: Option<ContainerState>,
}

#[derive(Debug, Deserialize, Default)]
struct ContainerState {
    #[serde(default)]
    waiting: Option<ContainerStateDetail>,
    #[serde(default)]
    terminated: Option<ContainerStateDetail>,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize, Default)]
struct ContainerStateDetail {
    #[serde(default)]
    reason: String,
    #[serde(default)]
    message: String,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize, Default)]
struct DeploymentResource {
    #[serde(default)]
    metadata: Metadata,
    #[serde(default)]
    spec: DeploymentSpec,
    #[serde(default)]
    status: DeploymentStatus,
}

#[derive(Debug, Deserialize, Default)]
struct DeploymentSpec {
    #[serde(default)]
    replicas: Option<u32>,
}

#[derive(Debug, Deserialize, Default)]
struct DeploymentStatus {
    #[serde(default)]
    replicas: Option<u32>,
    #[serde(default, rename = "readyReplicas")]
    ready_replicas: Option<u32>,
    #[serde(default, rename = "availableReplicas")]
    available_replicas: Option<u32>,
    #[serde(default, rename = "unavailableReplicas")]
    unavailable_replicas: Option<u32>,
}

#[derive(Debug, PartialEq, Eq)]
enum PodClassification {
    Healthy,
    Warning,
    Failing,
}

fn pod_waiting_reasons(pod: &PodResource) -> Vec<String> {
    let mut reasons = Vec::new();

    for statuses in [
        &pod.status.container_statuses,
        &pod.status.init_container_statuses,
    ] {
        for container in statuses {
            if let Some(state) = &container.state {
                if let Some(waiting) = &state.waiting {
                    reasons.push(waiting.reason.clone());
                }
            }
        }
    }

    reasons
}

fn classify_pod(pod: &PodResource) -> PodClassification {
    let phase = pod.status.phase.trim();
    let phase_lower = phase.to_ascii_lowercase();
    let waiting_reasons = pod_waiting_reasons(pod);

    let has_completed_termination = [
        &pod.status.container_statuses,
        &pod.status.init_container_statuses,
    ]
    .into_iter()
    .flatten()
    .any(|container| {
        container
            .state
            .as_ref()
            .and_then(|state| state.terminated.as_ref())
            .is_some_and(|terminated| terminated.reason.eq_ignore_ascii_case("Completed"))
    });

    let ready = pod.status.conditions.iter().any(|condition| {
        condition.type_.eq_ignore_ascii_case("Ready")
            && condition.status.eq_ignore_ascii_case("True")
    }) || (!pod.status.container_statuses.is_empty()
        && pod
            .status
            .container_statuses
            .iter()
            .all(|container| container.ready));

    if phase_lower == "succeeded" || has_completed_termination {
        return PodClassification::Healthy;
    }

    if ready && (phase_lower == "running" || phase_lower == "active") {
        return PodClassification::Healthy;
    }

    if phase_lower == "failed"
        || waiting_reasons.iter().any(|reason| {
            matches!(
                reason.to_ascii_lowercase().as_str(),
                "crashloopbackoff"
                    | "imagepullbackoff"
                    | "errimagepull"
                    | "invalidimage"
                    | "createcontainerconfigerror"
                    | "createcontainererror"
                    | "runcontainererror"
            )
        })
    {
        return PodClassification::Failing;
    }

    if phase_lower == "pending"
        || phase_lower == "unknown"
        || waiting_reasons.iter().any(|reason| {
            matches!(
                reason.to_ascii_lowercase().as_str(),
                "containercreating" | "podinitializing" | "waiting" | "createcontainerconfigerror"
            )
        })
    {
        return PodClassification::Warning;
    }

    if ready {
        PodClassification::Healthy
    } else if phase_lower == "running" {
        PodClassification::Warning
    } else {
        PodClassification::Failing
    }
}

fn pod_issues(output: &str) -> Result<Vec<String>> {
    let list: KubernetesList<PodResource> = serde_json::from_str(output)
        .map_err(|err| anyhow!("failed to parse pod JSON for issues: {err}"))?;

    let mut issues = Vec::new();
    for pod in &list.items {
        match classify_pod(pod) {
            PodClassification::Failing => {
                issues.push(format!(
                    "{}: {}",
                    pod.metadata.name,
                    pod.status.phase.trim()
                ));
            }
            PodClassification::Warning => {
                if pod_waiting_reasons(pod).into_iter().any(|reason| {
                    matches!(
                        reason.to_ascii_lowercase().as_str(),
                        "waiting" | "containercreating" | "podinitializing"
                    )
                }) {
                    issues.push(format!("{}: pending", pod.metadata.name));
                }
            }
            PodClassification::Healthy => {}
        }
    }

    Ok(issues)
}

pub fn summarize_pods(output: &str) -> Result<PodStatusSummary> {
    let list: KubernetesList<PodResource> =
        serde_json::from_str(output).map_err(|err| anyhow!("failed to parse pod JSON: {err}"))?;

    let mut summary = PodStatusSummary::default();
    for pod in &list.items {
        summary.total += 1;
        match classify_pod(pod) {
            PodClassification::Healthy => summary.healthy += 1,
            PodClassification::Warning => summary.warning += 1,
            PodClassification::Failing => summary.failing += 1,
        }
    }
    Ok(summary)
}

pub fn summarize_deployments(output: &str) -> Result<DeploymentStatusSummary> {
    let list: KubernetesList<DeploymentResource> = serde_json::from_str(output)
        .map_err(|err| anyhow!("failed to parse deployment JSON: {err}"))?;

    let mut summary = DeploymentStatusSummary::default();
    for deployment in &list.items {
        summary.total += 1;

        let desired = deployment
            .spec
            .replicas
            .or(deployment.status.replicas)
            .unwrap_or(0);
        let ready = deployment.status.ready_replicas.unwrap_or(0);
        let available = deployment.status.available_replicas.unwrap_or(0);
        let unavailable = deployment.status.unavailable_replicas.unwrap_or(0);

        if desired == 0 {
            if ready == 0 && available == 0 && unavailable == 0 {
                summary.ready += 1;
            } else if available > 0 {
                summary.partial += 1;
            } else {
                summary.unavailable += 1;
            }
        } else if available >= desired && unavailable == 0 {
            summary.ready += 1;
        } else if available > 0 || ready > 0 {
            summary.partial += 1;
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
        let exec = MockExecutor::new(
            r#"{"items":[{"metadata":{"name":"api"},"status":{"phase":"Running","containerStatuses":[{"ready":true,"state":{"running":{}}}]}}]}"#,
        );
        let context = AppContext {
            namespace: Some("payments".to_string()),
            kube_context: Some("prod".to_string()),
        };

        let result = show_pods(&exec, context);
        assert!(result.is_ok());
        assert_eq!(
            exec.args_for_last_call(),
            vec![
                "--context",
                "prod",
                "-n",
                "payments",
                "get",
                "pods",
                "-o",
                "json"
            ]
        );
    }

    #[test]
    fn summarize_pods_counts_healthy_warning_and_failing_resources() {
        let output = r#"{
            "items": [
                {"metadata":{"name":"api"},"status":{"phase":"Running","conditions":[{"type":"Ready","status":"True"}],"containerStatuses":[{"ready":true,"state":{"running":{}}}]}}
                ,
                {"metadata":{"name":"web"},"status":{"phase":"Running","containerStatuses":[{"ready":false,"state":{"waiting":{"reason":"CrashLoopBackOff"}}}]}}
                ,
                {"metadata":{"name":"worker"},"status":{"phase":"Pending","containerStatuses":[{"ready":false,"state":{"waiting":{"reason":"ContainerCreating"}}}]}}
                ,
                {"metadata":{"name":"db"},"status":{"phase":"Succeeded","containerStatuses":[{"ready":false,"state":{"terminated":{"reason":"Completed"}}}]}}
            ]
        }"#;

        let summary = summarize_pods(output).expect("pod summary should parse");
        assert_eq!(summary.total, 4);
        assert_eq!(summary.healthy, 2);
        assert_eq!(summary.warning, 1);
        assert_eq!(summary.failing, 1);
    }

    #[test]
    fn summarize_pods_rejects_malformed_json() {
        let result = summarize_pods("{not valid json");
        assert!(result.is_err());
    }

    #[test]
    fn summarize_deployments_counts_ready_partial_and_unavailable_replicas() {
        let output = r#"{
            "items": [
                {"metadata":{"name":"api"},"spec":{"replicas":2},"status":{"readyReplicas":2,"availableReplicas":2,"unavailableReplicas":0}},
                {"metadata":{"name":"web"},"spec":{"replicas":2},"status":{"readyReplicas":1,"availableReplicas":1,"unavailableReplicas":1}},
                {"metadata":{"name":"worker"},"spec":{"replicas":1},"status":{"readyReplicas":0,"availableReplicas":0,"unavailableReplicas":1}}
            ]
        }"#;

        let summary = summarize_deployments(output).expect("deployments summary should parse");
        assert_eq!(summary.total, 3);
        assert_eq!(summary.ready, 1);
        assert_eq!(summary.partial, 1);
        assert_eq!(summary.unavailable, 1);
    }

    #[test]
    fn summarize_deployments_allows_empty_lists() {
        let summary = summarize_deployments(r#"{"items":[]}"#).expect("empty list should parse");
        assert_eq!(summary.total, 0);
        assert_eq!(summary.ready, 0);
        assert_eq!(summary.partial, 0);
        assert_eq!(summary.unavailable, 0);
    }
}
