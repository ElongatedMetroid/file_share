#![feature(buf_read_has_data_left)]
use std::net::{TcpListener, TcpStream};

use file_share::Share;

fn main() {
    let listener = TcpListener::bind("127.0.0.1:34254").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                println!("Client {:?} connected", stream.peer_addr());
                handle_client(stream);
            },
            Err(error) => {
                eprintln!("Connection to client failed: {error}");
                continue;
            }
        }
    }
}

/// Only the official client will work for the most part so the server wont have
/// to handle additional things like making sure your command was correct (this
/// is checked on the official client)
fn handle_client(stream: TcpStream) {
    let share = match Share::read_from_stream(stream) {
        Ok(share) => share,
        Err(error) => {
            eprintln!("{error}");
            return;
        }
    };

    println!("{:#?}", share);
}