use crate::config;
use crate::tcp::get_response;
use crate::tcp::setup_stream;
use crate::types::RigInfo;
use log::warn;
use std::net::TcpStream;
use std::sync::mpsc::Sender;
use std::thread;
use std::time::{Duration, Instant};

pub fn fetch(r: &config::Rig, tx: &Sender<RigInfo>) -> Result<String, String> {
    let mut stream = setup_stream(&r.address)?;

    let interval = Duration::from_millis(300);
    let mut last_freq = 0;
    let mut last_mode = "".to_string();
    let mut last_pwr = None;
    let mut err_count = 0;
    loop {
        let start = Instant::now();

        let new_freq = match get_frequency(&mut stream) {
            Ok(f) => f,
            Err(e) => {
                warn!("could not read freq! {}", e);
                err_count += 1;
                continue;
            }
        };
        let new_mode = match get_mode(&mut stream) {
            Ok(f) => f,
            Err(e) => {
                warn!("could not read mode! {}", e);
                err_count += 1;
                continue;
            }
        };
        let new_pwr = match get_power(&mut stream, r.power_scale, r.send_power) {
            Ok(f) => f,
            Err(e) => {
                warn!("could not read power! {}", e);
                err_count += 1;
                continue;
            }
        };
        if last_freq != new_freq || last_mode != new_mode || last_pwr != new_pwr {
            last_freq = new_freq;
            last_mode = new_mode.clone();
            last_pwr = new_pwr;
            let msg = RigInfo {
                name: r.name.to_string(),
                freq: new_freq,
                mode: new_mode,
                power: new_pwr,
            };
            if let Err(e) = tx.send(msg) {
                return Err(format!("connection closed: {}", e));
            };
        }

        if let Some(rem) = interval.checked_sub(start.elapsed()) {
            thread::sleep(rem);
        }
        if err_count > 10 {
            return Err("too many errors".to_string());
        }
    }
}

fn get_frequency(stream: &mut TcpStream) -> Result<u64, String> {
    get_response(stream, "f\n")
        .map_err(|e| format!("get frequency failed: {e}"))?
        .trim()
        .parse::<u64>()
        .map_err(|e| format!("Could not parse frequency {}", e))
}

fn get_mode(stream: &mut TcpStream) -> Result<String, String> {
    get_response(stream, "m\n")
        .map_err(|e| format!("get mode failed: {e}"))
        .map(|s| s.trim().to_string())
}

fn get_power(stream: &mut TcpStream, scale: u64, enable: bool) -> Result<Option<u64>, String> {
    if !enable {
        return Ok(None);
    }
    let p = get_response(stream, "l RFPOWER\n")
        .map_err(|e| format!("get power failed: {e}"))?
        .trim()
        .parse::<f32>()
        .map_err(|e| format!("Could not parse power {}", e))?;

    let prod_f = p * scale as f32;
    let truncated = prod_f.trunc();

    let result = if truncated.is_nan() || truncated < 0.0 {
        0
    } else if truncated > u64::MAX as f32 {
        u64::MAX
    } else {
        truncated as u64
    };
    Ok(Some(result))
}
