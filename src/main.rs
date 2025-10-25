mod rigctl;

fn main() {
    let server_address = "127.0.0.1:4532";
    match rigctl::fetch(server_address) {
        Ok(success_message) => println!("{}", success_message),
        Err(error_message) => eprintln!("Error: {}", error_message),
    }
}

#[cfg(test)]
mod test {}
