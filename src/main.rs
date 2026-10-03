mod config;
mod rigctl;
mod wavelog;

use crate::config::load_config;
use clap::Parser;
use env_logger;
use log::{error, warn};
use std::path::PathBuf;
use std::process;
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

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

    env_logger::init();

    let cfg = match load_config(cli.config) {
        Ok(cfg) => cfg,
        Err(e) => {
            error!("could not load config: {}", e);
            process::exit(1)
        }
    };
    println!("{:#?}", cfg);
    let wavelog_address = cfg.wavelog.address;
    let token = cfg.wavelog.token;
    let mut handles = Vec::new();

    const MIN_RETRY: Duration = Duration::from_secs(10);
    const MAX_RETRY: Duration = Duration::from_secs(120);
    // a fetch that ran at least this long was actually delivering data, not just stuck retrying
    const HEALTHY_RUN: Duration = Duration::from_secs(5);

    let (tx, rx) = mpsc::channel::<types::RigInfo>();
    for r in cfg.rigs {
        let name = r.name.clone();
        let handle = thread::spawn({
            let tx = tx.clone();
            move || {
                let mut retry_delay = MIN_RETRY;
                loop {
                    let attempt_start = Instant::now();
                    if let Err(e) = rigctl::fetch(&r, &tx) {
                        warn!(
                            "{name}: fetch error {e}, retrying in {}s",
                            retry_delay.as_secs()
                        );
                    }
                    retry_delay = if attempt_start.elapsed() >= HEALTHY_RUN {
                        MIN_RETRY
                    } else {
                        (retry_delay * 2).min(MAX_RETRY)
                    };
                    thread::sleep(retry_delay);
                }
            }
        });
        handles.push(handle);
    }
    drop(tx);
    for info in rx {
        if let Err(e) = wavelog::send(&wavelog_address, &token, info) {
            warn!("could not send to wavelog: {}", e);
        }
    }

    for h in handles {
        let _ = h.join();
    }
}

#[cfg(test)]
mod test {}
