use serde::Deserialize;
use std::{fs, path::Path};

#[derive(Debug, Deserialize)]
pub struct Config {
    pub rigs: Vec<Rig>,
    pub wavelog: Wavelog,
}

#[derive(Debug, Deserialize)]
pub struct Rig {
    pub name: String,
    pub address: String, // keep as String; parse later if needed
    pub power_scale: u64,
    pub send_power: bool,
}

#[derive(Debug, Deserialize)]
pub struct Wavelog {
    pub address: String,
    pub token: String,
}

pub fn load_config<P: AsRef<Path>>(path: P) -> Result<Config, String> {
    let text = fs::read_to_string(path).map_err(|e| e.to_string())?;
    let cfg: Config = yaml_serde::from_str(&text).map_err(|e| e.to_string())?;
    Ok(cfg)
}
