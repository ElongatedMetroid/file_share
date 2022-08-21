use std::{net::TcpStream, process, io};

use retry::{delay::Fixed, retry_with_index};

use file_share::{Command, Share};

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
        buf.clear();

        println!("Enter what you would like to do, run HELP for help");

        io::stdin().read_line(&mut buf).unwrap_or_default();

        let command = match Command::parse(buf.as_str()) {
            Ok(command) => command,
            Err(error) => {
                eprintln!("Please type a correct command, or HELP for help: {error}");
                continue;
            }
        };

        if command.command_type().is_client() {
            command.execute_client_side().unwrap();
            continue;
        }

        let mut share = Share::new(command);

        // Load files into the vector, or text into the string
        match share.prepare_data() {
            Ok(_) => (),
            Err(error) => {
                eprintln!("Error occured while preparing data: {error}");
                continue;
            }
        }
    
        match share.write_to_stream(&mut stream) {
            Ok(_) => (),
            Err(error) => {
                eprintln!("Unresolvable error: {error}");
                process::exit(1);
            }
        }
    }
}
