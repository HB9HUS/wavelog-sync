use crate::tcp::get_response;
use crate::tcp::setup_stream;
use crate::types::RigInfo;
use std::net::TcpStream;
use std::str;
use std::sync::mpsc::Sender;
use std::thread;
use std::time::{Duration, Instant};

pub fn fetch(name: &str, address: &str, tx: &Sender<RigInfo>) -> Result<String, String> {
    let mut stream = setup_stream(address)?;

    let interval = Duration::from_millis(300);
    let mut last_freq = 0;
    let mut last_mode = "".to_string();
    let mut err_count = 0;
    loop {
        let start = Instant::now();

        let res = (get_frequency(&mut stream), get_mode(&mut stream));
        match res {
            (Ok(f), Ok(m)) => {
                if last_freq != f || last_mode != m {
                    last_freq = f;
                    last_mode = m.clone();
                    let msg = RigInfo {
                        name: name.to_string(),
                        freq: f,
                        mode: m,
                    };
                    if let Err(e) = tx.send(msg) {
                        return Err(format!("connection closed: {}", e));
                    };
                }
            }
            (_, _) => {
                if let Err(e) = res.0 {
                    eprintln!("could not read freq! {}", e);
                    err_count += 1;
                }
                if let Err(e) = res.1 {
                    eprintln!("could not read mode! {}", e);
                    err_count += 1;
                }
            }
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
