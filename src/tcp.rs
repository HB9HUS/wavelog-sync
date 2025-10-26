use std::io::{self, BufRead, BufReader, Write};
use std::net::TcpStream;
use std::str;
use std::time::Duration;

pub fn setup_stream(address: &str) -> Result<TcpStream, String> {
    let stream = TcpStream::connect(address).map_err(|e| format!("connect failed: {e}"))?;
    stream
        .set_read_timeout(Some(Duration::from_millis(900)))
        .map_err(|e| format!("set_read_timeout failed: {e}"))?;
    stream
        .set_write_timeout(Some(Duration::from_millis(900)))
        .map_err(|e| format!("set_write_timeout failed: {e}"))?;
    return Ok(stream);
}

pub fn get_response(stream: &mut TcpStream, message: &str) -> Result<String, String> {
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
