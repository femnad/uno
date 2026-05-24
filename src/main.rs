use clap::{Args, Command, CommandFactory, Parser, Subcommand};
use clap_complete::{Generator, Shell, generate};
use std::io;

mod latest_kernel;
mod prefix;

#[derive(Debug, Parser)]
#[command(
    name = "uno",
    version,
    about = "Do everything*",
    long_about = "*Do some of the things"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Args, Debug)]
struct GenerateArgs {
    #[arg(help = "Shell name")]
    shell: Shell,
}

#[derive(Args, Debug)]
struct LatestKernelArgs {
    #[arg(short, long, help = "Displays latest and running kernel versions")]
    print: bool,
}

#[derive(Args, Debug)]
struct PrefixArgs {
    #[arg(short, long, help = "Copy the path to clipboard")]
    copy: bool,
    #[arg(help = "Path relative to the current path")]
    relative_path: Option<String>,
}

#[derive(Debug, Subcommand)]
enum Commands {
    #[command(about = "Generate completions")]
    Generate(GenerateArgs),
    #[command(about = "Check if latest kernel is running", alias = "lk")]
    LatestKernel(LatestKernelArgs),
    #[command(about = "Show prefix within repo", alias = "prf")]
    Prefix(PrefixArgs),
}

fn print_completions<G: Generator>(generator: G, cmd: &mut Command) {
    generate(
        generator,
        cmd,
        cmd.get_name().to_string(),
        &mut io::stdout(),
    );
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Generate(generate_args) => {
            print_completions(generate_args.shell, &mut Cli::command())
        }
        Commands::LatestKernel(latest_kernel_args) => {
            latest_kernel::check(latest_kernel_args.print);
        }
        Commands::Prefix(prefix_args) => {
            prefix::get(prefix_args.relative_path.clone(), prefix_args.copy)
        }
    }
}
