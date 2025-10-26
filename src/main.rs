mod config;
mod rigctl;
mod wavelog;

use crate::config::load_config;
use clap::Parser;
use std::path::PathBuf;
use std::process;
use std::sync::mpsc;
use std::thread;

#[derive(Parser, Debug)]
#[command(name = "myapp", version, about = "Example app")]
struct Cli {
    /// Path to config file
    #[arg(short = 'c', long = "config", default_value = "config.yaml")]
    config: PathBuf,
}

pub mod tcp;
pub mod types;

fn main() {
    let cli = Cli::parse();

    let cfg = match load_config(cli.config) {
        Ok(cfg) => cfg,
        Err(e) => {
            eprintln!("could not load config: {}", e);
            process::exit(1)
        }
    };
    println!("{:#?}", cfg);
    let wavelog_address = cfg.wavelog.address;
    let token = cfg.wavelog.token;
    let mut handles = Vec::new();

    let (tx, rx) = mpsc::channel::<types::RigInfo>();
    for r in &cfg.rigs {
        let handle = thread::spawn({
            let tx = tx.clone();
            let name = r.name.clone();
            let address = r.address.clone();
            move || {
                if let Err(e) = rigctl::fetch(name, &address, tx) {
                    eprintln!("fetch error: {}", e);
                }
            }
        });
        handles.push(handle);
    }
    drop(tx);
    for info in rx {
        if let Err(e) = wavelog::send(&wavelog_address, &token, info) {
            eprintln!("could not send to wavelog: {}", e);
        }
    }

    for h in handles {
        let _ = h.join();
    }
}

#[cfg(test)]
mod test {}
