use std::{net::TcpStream, process, io::{self, Write}};

use retry::{delay::Fixed, retry_with_index};

use file_share::{Command};

fn main() {
    let stream = 
    // Retry connecting to the server 10 times, once every 1000 milliseconds
    retry_with_index(Fixed::from_millis(1000).take(9), |current_try| {
        match TcpStream::connect("127.0.0.1:34254") {
            Ok(stream) => Ok(stream),
            Err(error) => {
                eprintln!("Connection to server failed, try: {current_try}");
                Err(error)
            },
        }
    });

    let stream = stream.unwrap_or_else(|error| {
        eprintln!("Failed to connect to server!: {error}");
        process::exit(1)
    });

    handle_connection(stream);
}

fn handle_connection(mut stream: TcpStream) {
    println!("Connected to the server!");
    let mut buf = String::new();

    loop {
        println!("Enter what you would like to do, run HELP for help");

        io::stdin().read_line(&mut buf).unwrap_or_default();

        let command = match Command::parse(buf.as_str()) {
            Ok(command) => command,
            Err(error) => {
                eprintln!("Please type a correct command, or HELP for help: {error}");
                continue;
            }
        };
    
        let command = bincode::serialize(&command).unwrap();

        stream.write_all(&command[..]).unwrap_or_else(|error| {
            eprintln!("Failed to write command to tcp stream: {error}");
            process::exit(1);
        });

        buf.clear();
    }
}
