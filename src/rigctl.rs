use std::io::{self, BufRead, BufReader, Write};
use std::net::TcpStream;
use std::str;
use std::thread;
use std::time::{Duration, Instant};

pub fn fetch(address: &str) -> Result<String, String> {
    let mut stream = TcpStream::connect(address).map_err(|e| format!("connect failed: {e}"))?;
    stream
        .set_read_timeout(Some(Duration::from_millis(900)))
        .map_err(|e| format!("set_read_timeout failed: {e}"))?;
    stream
        .set_write_timeout(Some(Duration::from_millis(900)))
        .map_err(|e| format!("set_write_timeout failed: {e}"))?;

    let intervall = Duration::from_secs(10);
    loop {
        let start = Instant::now();

        match get_frequency(&mut stream) {
            Ok(f) => {
                println!("got frequency: {}", f)
            }
            Err(e) => {
                eprintln!("could not read frequency! {}", e);
            }
        }

        match get_mode(&mut stream) {
            Ok(f) => {
                println!("got mode: {}", f)
            }
            Err(e) => {
                eprintln!("could not read mode! {}", e);
            }
        }

        let elapsed = start.elapsed();
        if elapsed < intervall {
            thread::sleep(intervall - elapsed);
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
