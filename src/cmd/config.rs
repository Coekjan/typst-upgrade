use std::fs;

use clap::Args;

use crate::global::{Config, CONFIG, CONFIG_PATH};

#[derive(Args)]
pub struct ConfigArgs {
    #[clap(short, long)]
    list: bool,

    #[clap(flatten)]
    config: Config,
}

pub fn execute(args: ConfigArgs) {
    let path = CONFIG_PATH.get().unwrap();

    fs::write(
        path,
        toml::to_string(&CONFIG.get().unwrap().merge(&args.config)).unwrap(),
    )
    .unwrap();

    if args.list {
        print!("{}", fs::read_to_string(path).unwrap());
    }
}
