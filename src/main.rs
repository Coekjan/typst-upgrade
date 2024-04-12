use std::{
    env,
    path::{Path, PathBuf},
};

use clap::{Parser, Subcommand};
use cmd::{config::ConfigArgs, run::RunArgs};

mod cmd;
mod global;
mod typstdep;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[arg(
        short,
        long,
        value_name = "FILE",
        env = "TYPST_UPGRADE_CONFIG",
        default_value = Path::new(&env::var("HOME").unwrap())
            .join(".config")
            .join("typst-upgrade.toml")
            .into_os_string(),
    )]
    config: PathBuf,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Config(ConfigArgs),
    Run(RunArgs),
}

fn main() {
    let cli = Cli::parse();

    global::config_load(cli.config);

    match cli.command {
        Commands::Config(args) => cmd::config::execute(args),
        Commands::Run(args) => cmd::run::execute(args),
    }
}
