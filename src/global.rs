use std::{
    fs::{self, File},
    path::{Path, PathBuf},
    sync::OnceLock,
};

use serde::{Deserialize, Serialize};

pub static CONFIG: OnceLock<Config> = OnceLock::new();

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub token: Option<String>,
}

pub fn config_load(path: PathBuf) {
    if !Path::new(&path).exists() {
        File::create(&path).unwrap();
    }

    CONFIG
        .set(toml::from_str(&fs::read_to_string(path).unwrap()).unwrap())
        .unwrap();
}
