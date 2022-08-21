use std::{net::{TcpListener, TcpStream}, io::{Write, Read}};

use file_share::Command;

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

fn handle_client(mut stream: TcpStream) {
    // Read command `UPLOAD text.txt` `RECIEVE text.txt` `CATALOGUE`
    // stream.read(buf);

    // parse buf (Syntax checked on client side)

    // let send = match 

    // stream.write_all(send)

    let mut buf = Vec::new();
    stream.read_to_end(&mut buf).unwrap();
    
    let command = bincode::deserialize::<Command>(&buf[..]).unwrap();

    println!("{:#?}", command);
}