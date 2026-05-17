use clap::{Args, Parser, Subcommand};

mod latest_kernel;
mod prefix;

#[derive(Debug, Parser)]
#[command(name = "uno", version, about = "Do everything*", long_about = "*Do some of the things")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Args, Debug)]
struct PrefixArgs {
    #[arg(help = "Path relative to the current path")]
    relative_path: Option<String>,
}

#[derive(Debug, Subcommand)]
enum Commands {
    #[command(about = "Check if latest kernel is running", alias = "lk")]
    LatestKernel,
    #[command(about = "Show prefix within repo", alias = "prf")]
    Prefix(PrefixArgs),
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::LatestKernel => { latest_kernel::check()}
        Commands::Prefix(prefix_args) => prefix::get(prefix_args.relative_path.clone()),
    }
}
