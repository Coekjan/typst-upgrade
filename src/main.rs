use std::path::PathBuf;

use clap::{Parser, Subcommand};
use cmd::run::RunArgs;

mod cmd;
mod typstdep;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[arg(short, long, value_name = "FILE", env = "TYPST_UPGRADE_CONFIG")]
    config: Option<PathBuf>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Config,
    Run(RunArgs),
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Config => todo!(),
        Commands::Run(args) => {
            cmd::run::execute(args);
        }
    }
}
