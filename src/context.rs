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
}
