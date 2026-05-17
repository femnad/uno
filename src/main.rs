use clap::{Args, Parser, Subcommand};

mod prefix;

#[derive(Debug, Parser)]
#[command(name = "myapp", version, about = "Do everything*", long_about = "*Do some of the things")]
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
    #[command(alias = "prf", about = "Show prefix within repo")]
    Prefix(PrefixArgs),
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Prefix(prefix_args) => prefix::get(prefix_args.relative_path.clone()),
    }
}
