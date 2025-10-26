mod rigctl;
use std::sync::mpsc;
use std::thread;

pub mod types;

fn main() {
    let server_address = "127.0.0.1:4532";
    let (tx, rx) = mpsc::channel::<types::RigInfo>();
    let handle = thread::spawn({
        let tx = tx.clone();
        move || {
            if let Err(e) = rigctl::fetch(server_address, tx) {
                eprintln!("fetch error: {}", e);
            }
        }
    });
    for info in rx {
        println!("freq={} mode={}", info.freq, info.mode);
    }

    let _ = handle.join();
}

#[cfg(test)]
mod test {}
