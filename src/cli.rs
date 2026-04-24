use clap::{Parser, Subcommand};

pub mod pods;
pub mod logs;
pub mod ctx;
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