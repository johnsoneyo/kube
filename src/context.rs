#[allow(dead_code)]
pub struct AppContext {
    pub namespace: Option<String>,
    pub kube_context: Option<String>,
}

impl AppContext {
    pub fn kubectl_base_args(&self) -> Vec<String> {
        let mut args = vec![];

        if let Some(ctx) = &self.kube_context {
            args.push("--context".into());
            args.push(ctx.clone());
        }

        if let Some(ns) = &self.namespace {
            args.push("-n".into());
            args.push(ns.clone());
        }

        args
    }

    pub fn kubectl_status_args(&self, resource: &str) -> Vec<String> {
        let mut args = self.kubectl_base_args();

        if self.namespace.is_none() {
            args.push("-A".to_string());
        }

        args.push("get".to_string());
        args.push(resource.to_string());
        args.push("-o".to_string());
        args.push("json".to_string());
        args
    }
}

#[cfg(test)]
mod tests {
    use super::AppContext;

    #[test]
    fn status_scope_uses_all_namespaces_when_no_namespace_is_set() {
        let context = AppContext {
            namespace: None,
            kube_context: Some("prod".to_string()),
        };

        let args = context.kubectl_status_args("pods");
        assert_eq!(
            args,
            vec!["--context", "prod", "-A", "get", "pods", "-o", "json"]
        );
    }

    #[test]
    fn status_scope_uses_namespace_when_one_is_provided() {
        let context = AppContext {
            namespace: Some("payments".to_string()),
            kube_context: Some("prod".to_string()),
        };

        let args = context.kubectl_status_args("deployments");
        assert_eq!(
            args,
            vec![
                "--context",
                "prod",
                "-n",
                "payments",
                "get",
                "deployments",
                "-o",
                "json"
            ]
        );
    }
}
