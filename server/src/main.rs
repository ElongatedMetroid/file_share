#![feature(buf_read_has_data_left)]
use std::{net::{TcpListener, TcpStream}, io::Write};

use file_share::{Share, Location};

mod threadpool;

use threadpool::ThreadPool;

fn main() {
    let pool = ThreadPool::build(5).unwrap();

    let listener = TcpListener::bind("127.0.0.1:34254").unwrap();

    for stream in listener.incoming() {
        let stream = match stream {
            Ok(stream) => {
                println!("Client {:?} connected", stream.peer_addr());
                stream
            },
            Err(error) => {
                eprintln!("Connection to client failed: {error}");
                continue;
            }
        };

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
        let mut share = match Share::read_from_stream(&mut stream, Location::Server) {
            Ok(share) => share,
            Err(error) => {
                eprintln!("{error}");
                return;
            }
        };
    
        stream.flush().unwrap();
    
        if let Err(e) = share.execute() {
            share.set_error_response(e);
        };
    
        share.write_to_stream(&mut stream, Location::Server).unwrap();
    }
}