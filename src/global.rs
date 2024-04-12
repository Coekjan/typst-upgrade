use std::{
    fmt::Display,
    fs::{self, File},
    path::{Path, PathBuf},
    sync::OnceLock,
};

use clap::Args;
use serde::{Deserialize, Serialize};

pub static CONFIG_PATH: OnceLock<PathBuf> = OnceLock::new();
pub static CONFIG: OnceLock<Config> = OnceLock::new();

#[derive(Args, Serialize, Deserialize, Debug)]
pub struct Config {
    #[clap(long)]
    pub token: Option<String>,
}

impl Display for Config {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", toml::to_string(self).map_err(|_| std::fmt::Error)?)
    }
}

impl Config {
    pub fn merge(&self, other: &Self) -> Self {
        // TODO: Use macro to generate this code
        Self {
            token: other.token.clone().or_else(|| self.token.clone()),
        }
    }
}

pub fn config_load(path: PathBuf) {
    if !Path::new(&path).exists() {
        File::create(&path).unwrap();
    }
    let path = fs::canonicalize(path).unwrap();

    CONFIG
        .set(toml::from_str(&fs::read_to_string(&path).unwrap()).unwrap())
        .unwrap();
    CONFIG_PATH.set(path).unwrap();
}
