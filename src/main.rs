mod cli;
mod context;
mod di;


use clap::Parser;
use cli::{Cli, Commands};
use crate::context::AppContext;
use crate::di::Kubectl;

fn main() {
    let cli = Cli::parse();

    let ctx = AppContext {
        namespace: cli.namespace,
        kube_context: cli.context,
    };

    let exec = Kubectl;

    match cli.command {

        Commands::Pods { command } => {
            cli::pods::handle(command, &exec, ctx);
        },
        Commands::Logs { command } => {
            cli::logs::handle(command, &exec, ctx);
        }
        //
        // Commands::Ctx { command } => {
        //     cli::ctx::handle(command);
        // }
        //
        // Commands::Status { command } => {
        //     cli::status::handle(command);
        // }
        _ => {println!("wip")}
    }
   
}