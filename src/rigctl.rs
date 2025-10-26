use crate::tcp::get_response;
use crate::tcp::setup_stream;
use crate::types::RigInfo;
use std::net::TcpStream;
use std::str;
use std::sync::mpsc::Sender;
use std::thread;
use std::time::{Duration, Instant};

pub fn fetch(name: String, address: &str, tx: Sender<RigInfo>) -> Result<String, String> {
    let mut stream = setup_stream(address)?;

    let interval = Duration::from_secs(10);
    loop {
        let start = Instant::now();

        let res = (get_frequency(&mut stream), get_mode(&mut stream));
        match res {
            (Ok(f), Ok(m)) => {
                let msg = RigInfo {
                    name: name.to_string(),
                    freq: f,
                    mode: m,
                };
                if let Err(e) = tx.send(msg) {
                    return Err(format!("connection closed: {}", e));
                };
            }
            (_, _) => {
                if let Err(e) = res.0 {
                    eprintln!("could not read freq! {}", e);
                }
                if let Err(e) = res.1 {
                    eprintln!("could not read mode! {}", e);
                }
            }
        }

        if let Some(rem) = interval.checked_sub(start.elapsed()) {
            thread::sleep(rem);
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
