#![feature(buf_read_has_data_left)]
use std::{net::{TcpListener, TcpStream}, io::Write, process};

use file_share::{Share, Location};

mod threadpool;

use threadpool::ThreadPool;

fn main() {
    // Create a new thread pool
    let pool = match ThreadPool::build(5) {
        Ok(p) => p,
        // Thread pool could not be created
        Err(e) => {
            eprintln!("{e}");

            // Attempt to create a thread pool with a hardcoded value (error could be caused by creating a pool with zero threads, or
            // pottentialy creating to much threads).
            match ThreadPool::build(3) {
                Ok(p) => {
                    println!("Thread pool error fixed: created pool with 3 threads");
                    p
                },
                // Thread pool error was not resolved, exit program
                Err(e) => {
                    eprintln!("Could not resolve thread pool error: {e}");

                    process::exit(1);
                }
            }
        }
    };

    // Create a TcpListener and attempt to bind to the given ip
    let listener = match TcpListener::bind("127.0.0.1:34254") {
        // Binding to the address successs
        Ok(listener) => {
            println!("Listener bind success");
            listener
        },
        // Binding to address fail
        Err(error) => {
            // TODO: Attempt to connect to backup ip
            eprintln!("Failed in binding to address: {error}");
            process::exit(1);
        },
    };

    // Loop through each connection
    for stream in listener.incoming() {
        // Get the value inside stream
        let stream = match stream {
            // Connection success
            Ok(stream) => {
                println!("Client {:?} connected", stream.peer_addr());
                stream
            },
            // Conection failed
            Err(error) => {
                // Print log and continue
                eprintln!("Connection to client failed: {error}");
                continue;
            }
        };

        // Execute the handle_client() function for each connection
        pool.execute(|| {
            handle_client(stream)
        });
    }
}

/// Only the official client will work for the most part so the server wont have
/// to handle additional things like making sure your command was correct (this
/// is checked on the official client)
fn handle_client(mut stream: TcpStream) {
    loop {
        // Read data that was sent from client
        let mut share = match Share::read_from_stream(&mut stream, Location::Server) {
            // Successful read
            Ok(share) => share,
            // Invalid read
            Err(error) => {
                eprintln!("{error}");
                // Just returns since unofficial clients/requests wont be supported, no error is returned since with the official client
                // these will be checked on the client side, I dont want the server to have to use extra CPU power to check this and
                // return an error. This should not effect most people.
                return;
            }
        };

        // Execute the recieved command
        if let Err(e) = share.execute() {
            // If there was an error set the servers error response
            share.set_error_response(e);
        };

        // Write share to stream since we executed the command and all the data needed is inside
        match share.write_to_stream(&mut stream, Location::Server) {
            Ok(_) => (),
            Err(error) => {
                eprintln!("Failed to write to stream: {error}");
                return;
            }
        }

        stream.flush().unwrap_or_else(|error| {
            eprintln!("Failed to flush stream: {error}: Client ip {}", stream.peer_addr().unwrap());
        });
    }
}