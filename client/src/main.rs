use std::{net::TcpStream, process, io::{self, Write}};

use retry::{delay::Fixed, retry_with_index};

use file_share::{ShareCommand, Share, Location, Config};

fn main() {
    let config = Config::build("Config.toml").unwrap_or_else(|error| {
        eprintln!("Config build error: {error}");
        process::exit(1);
    }).client().unwrap_or_else(|error| {
        eprintln!("Config build error: {error}");
        process::exit(1);
    });

    let stream = 
    // Retry connecting to the server 10 times, once every 1000 milliseconds
    retry_with_index(Fixed::from_millis(config.retry_delay()).take(config.retry_amount()), |current_try| {
        match TcpStream::connect(config.server()) {
            Ok(stream) => Ok(stream),
            Err(error) => {
                eprintln!("Connection to server failed, attempt: {current_try}");
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
        // Empty the buffer
        buf.clear();

        println!("Enter what you would like to do, run HELP for help");

        // Read in the command
        io::stdin().read_line(&mut buf).unwrap_or_default();

        // Parse the command into a Command struct
        let command = match ShareCommand::parse(buf.as_str()) {
            // Command was successfully parsed
            Ok(command) => command,
            // Invalid command
            Err(error) => {
                eprintln!("Please type a correct command, or HELP for help: {error}");
                continue;
            }
        };

        // Create a new share with the command we got above
        let mut share = Share::new(command, Location::Client);

        // Prepare data (if needed) for the specified command
        if let Err(error) = share.prepare_data() {
            // The error message should be clear to the user (file not found, is a directory, etc.) but better error handling will be 
            // added later
            eprintln!("Error occured while preparing data: {error}");
            continue;
        }

        // Write the share we prepared to the server/stream
        share.write_to_stream(&mut stream, Location::Client).unwrap_or_else(|error| {
            // will handle these errors later.
            eprintln!("Error occurred: {error}");
            process::exit(1);
        });

        // Make sure all buffered contents reach there destination
        stream.flush().unwrap_or_else(|error| {
            // will handle these errors later.
            eprintln!("Error occurred: {error}");
            process::exit(1);
        });

        // Read in the response the server send, this can contain requested files, text data, etc.
        let mut server_response_share = Share::read_from_stream(&mut stream, Location::Client).unwrap_or_else(|error| {
            // will handle these errors later.
            eprintln!("Error occurred: {error}");
            process::exit(1);
        });

        // If needed execute instructions to get data from the Share struct to storage, print some text data, etc.
        server_response_share.execute().unwrap_or_else(|error| {
            // will handle these errors later.
            eprintln!("Error occurred: {error}");
            process::exit(1);
        });
    }
}
