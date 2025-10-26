use crate::types::RigInfo;
use std::io::{self, BufRead, BufReader, Write};
use std::net::TcpStream;
use std::str;
use std::sync::mpsc::Sender;
use std::thread;
use std::time::{Duration, Instant};

pub fn fetch(address: &str, tx: Sender<RigInfo>) -> Result<String, String> {
    let mut stream = TcpStream::connect(address).map_err(|e| format!("connect failed: {e}"))?;
    stream
        .set_read_timeout(Some(Duration::from_millis(900)))
        .map_err(|e| format!("set_read_timeout failed: {e}"))?;
    stream
        .set_write_timeout(Some(Duration::from_millis(900)))
        .map_err(|e| format!("set_write_timeout failed: {e}"))?;

    let interval = Duration::from_secs(10);
    loop {
        let start = Instant::now();

        let res = (get_frequency(&mut stream), get_mode(&mut stream));
        match res {
            (Ok(f), Ok(m)) => {
                let msg = RigInfo { freq: f, mode: m };
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

fn get_frequency(stream: &mut TcpStream) -> Result<i64, String> {
    get_response(stream, "f\n")
        .map_err(|e| format!("get frequency failed: {e}"))?
        .trim()
        .parse::<i64>()
        .map_err(|e| format!("Could not parse frequency {}", e))
}

fn get_mode(stream: &mut TcpStream) -> Result<String, String> {
    get_response(stream, "m\n")
        .map_err(|e| format!("get mode failed: {e}"))
        .map(|s| s.trim().to_string())
}

fn get_response(stream: &mut TcpStream, message: &str) -> Result<String, String> {
    stream
        .write_all(message.as_bytes())
        .map_err(|e| format!("could not send command! {}", e))?;
    stream.flush().map_err(|e| format!("flush failed: {e}"))?;

    let mut line = String::new();
    let mut reader = BufReader::new(stream.try_clone().map_err(|e| e.to_string())?);
    match reader.read_line(&mut line) {
        Ok(0) => Err("server closed connection".into()),
        Ok(_) => Ok(line),
        Err(e) if e.kind() == io::ErrorKind::TimedOut => Err(format!("read timed out")),
        Err(e) => Err(format!("unexpected error {}", e)),
    }
}
