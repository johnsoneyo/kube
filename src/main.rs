mod cli;
mod context;
mod di;

use crate::context::AppContext;
use crate::di::Kubectl;
use clap::Parser;
use cli::{Cli, Commands};

fn main() {
    if let Err(err) = run() {
        eprintln!("error: {err}");
        std::process::exit(1);
    }
}

fn run() -> anyhow::Result<()> {
    let cli = Cli::parse();

    let ctx = AppContext {
        namespace: cli.namespace,
        kube_context: cli.context,
    };

    let exec = Kubectl;

    match cli.command {
        Commands::Pods { command } => cli::pods::handle(command, &exec, ctx),
        Commands::Logs { command } => cli::logs::handle(command, &exec, ctx),
        Commands::Ctx { command } => cli::ctx::handle(command, &exec, ctx),
        Commands::Status { command } => cli::status::handle(command, &exec, ctx),
    }
}
