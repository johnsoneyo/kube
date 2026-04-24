use clap::Subcommand;

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