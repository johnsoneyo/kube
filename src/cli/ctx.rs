use clap::Subcommand;


#[derive(Subcommand, Debug)]
pub enum CtxCmd {
    Current,
    Use { name: String },
    List,
}

