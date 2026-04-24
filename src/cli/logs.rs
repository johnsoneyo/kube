use crate::cli::pods::{list_pods, PodsCmd};
use crate::context::AppContext;
use crate::di::{Executor, Kubectl};
use clap::Subcommand;

#[derive(Subcommand, Debug)]
pub enum LogsCmd {
    // simplest MVP
    Follow { name: String },
}

pub fn handle(cmd: LogsCmd, exec: &dyn Executor, _context: AppContext) {
    match cmd {
        LogsCmd::Follow { name } => {
            show_logs(exec, name, _context);
        }
    }
}

pub fn show_logs(exec: &dyn Executor, name: String, app_context: AppContext) {
    let mut args = app_context.kubectl_base_args();

    args.push("logs".to_string());
    args.push("-f".to_string());
    args.push(name);

    exec.stream(
        &args.iter().map(|s| s.as_str()).collect::<Vec<&str>>(),
        &mut |line| {
            if !line.starts_with("NAME") {
                println!("{line}");
            }
        },
    );
}
