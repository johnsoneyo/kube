use clap::{Parser, Subcommand};

pub mod ctx;
pub mod logs;
pub mod pods;
pub mod status;

#[derive(Parser, Debug)]
#[command(name = "kube")]
#[command(about = "Kubernetes companion CLI")]
pub struct Cli {
    /// The context in kubernetes
    #[arg(long)]
    pub context: Option<String>,

    /// The name space to be added implicitly
    #[arg(long)]
    pub namespace: Option<String>,

    /// Commands to run
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    Pods {
        #[command(subcommand)]
        command: pods::PodsCmd,
    },

    Logs {
        #[command(subcommand)]
        command: logs::LogsCmd,
    },

    Ctx {
        #[command(subcommand)]
        command: ctx::CtxCmd,
    },

    Status {
        #[command(subcommand)]
        command: status::StatusCmd,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cli_parses_context_namespace_and_pod_filter() {
        let cli = Cli::try_parse_from([
            "kube",
            "--context",
            "prod",
            "--namespace",
            "payments",
            "pods",
            "list",
            "api",
        ])
        .expect("cli should parse pod command with context and namespace");

        assert_eq!(cli.context.as_deref(), Some("prod"));
        assert_eq!(cli.namespace.as_deref(), Some("payments"));
        match cli.command {
            Commands::Pods { command } => match command {
                pods::PodsCmd::List { filter } => assert_eq!(filter.as_deref(), Some("api")),
            },
            _ => panic!("expected pods command"),
        }
    }

    #[test]
    fn cli_parses_ctx_current_command() {
        let cli =
            Cli::try_parse_from(["kube", "ctx", "current"]).expect("ctx command should parse");

        match cli.command {
            Commands::Ctx { command } => match command {
                ctx::CtxCmd::Current => {}
                _ => panic!("expected current context command"),
            },
            _ => panic!("expected ctx command"),
        }
    }
}
