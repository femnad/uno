use clap::{Args, Command, CommandFactory, Parser, Subcommand};
use clap_complete::{Generator, Shell, generate};
use std::io;

mod args;
mod internal;
mod latest_kernel;
mod prefix;
mod pwd;
mod qmk;

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
struct PwdArgs {
    #[arg(short, long, help = "Copy the path to clipboard")]
    copy: bool,
    #[arg(help = "Relate path to current dir to include")]
    path: Option<String>,
}

#[derive(Args, Debug)]
struct PrefixArgs {
    #[arg(short, long, help = "Copy the path to clipboard")]
    copy: bool,
    #[arg(help = "Path relative to the current path")]
    relative_path: Option<String>,
}

#[derive(Args, Debug)]
struct LayoutArgs {
    #[arg(help = "Keymap name")]
    keyboard: String,
    #[arg(short, long, default_value = "base.yml", help = "Config file")]
    config: String,
}

#[derive(Debug, Subcommand)]
enum QmkOp {
    #[command(about = "Write QMK mapping")]
    Layout(LayoutArgs)
}

#[derive(Args, Debug)]
#[command(about = "QMK operations")]
struct QmkArgs {
    #[command(subcommand)]
    qmk_op: QmkOp,
}

#[derive(Debug, Subcommand)]
enum Commands {
    #[command(about = "Generate completions")]
    Generate(GenerateArgs),
    #[command(about = "Check if latest kernel is running", alias = "lk")]
    LatestKernel(LatestKernelArgs),
    #[command(about = "Print or copy current path")]
    Pwd(PwdArgs),
    #[command(about = "Show prefix within repo", alias = "prf")]
    Prefix(PrefixArgs),
    #[command(about = "QMK operations")]
    Qmk(QmkArgs),
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
            latest_kernel::check(latest_kernel_args.print)
        }
        Commands::Pwd(pwd_args) => pwd::run(pwd_args.copy, pwd_args.path.clone()),
        Commands::Prefix(prefix_args) => {
            prefix::get(prefix_args.relative_path.clone(), prefix_args.copy)
        }
        Commands::Qmk(qmk_args) => {
            match &qmk_args.qmk_op {
                QmkOp::Layout(layout_args) => {
                    let parsed = args::Args{config: layout_args.config.clone(), keyboard: layout_args.keyboard.clone()};
                    qmk::run(parsed)
                }
            }
        }
    }
}
