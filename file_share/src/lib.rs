#![feature(core_intrinsics)]

use std::{process, fs::{File, self}, io::{Read, Write, BufReader, BufRead}, mem, net::TcpStream};

use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub enum CommandType {
    // Runs on client
    Exit,
    Help,

    // Runs on server
    Upload,
    Recieve,
    Catalogue,
}

impl CommandType {
    fn has_arg(&self) -> bool {
        if *self == CommandType::Exit ||
           *self == CommandType::Help ||
           *self == CommandType::Catalogue 
        {
            false
        } else {
            true
        }
    }
    /// Returns true if the command runs on the client side
    pub fn is_client(&self) -> bool {
        if *self == CommandType::Exit ||
           *self == CommandType::Help
        {
            true
        } else {
            false
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Command {
    command_type: CommandType,
    arg: Option<String>,
}

impl Command {
    pub fn parse(command: &str) -> Result<Command, &'static str> {
        if command.is_empty() {
            return Err("Parse error: Empty command");
        }

        let mut command_tokens = command.split_whitespace();

        let command_type = match command_tokens.next().unwrap() {
            "EXIT" => CommandType::Exit,
            "HELP" => CommandType::Help,

            "UPLOAD" => CommandType::Upload,
            "RECIEVE" => CommandType::Recieve,
            "CATALOGUE" => CommandType::Catalogue,

            _ => {
                return Err("Parse error: Unknown command type");
            }
        };

        let arg: Option<String> = match command_tokens.next() {
            Some(arg) => Some(arg.to_string()),
            // Command does not require an argument
            None if !command_type.has_arg() => None,
            // Command requires an argument
            None if command_type.has_arg() => {
                return Err("Parse error: No argument provided for command");
            },
            // ?
            None => {
                return Err("Parse error: Unknown error");
            },
        };

        let command = Command { 
            command_type, 
            arg, 
        };

        Ok(command)
    }
    pub fn command_type(&self) -> &CommandType {
        &self.command_type
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub enum Location {
    Client,
    Server,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Share {
    command: Command,

    /// Contains file data
    file: Option<Vec<u8>>,
    /// Contains text data, this is interpretted diferent ways depending on the
    /// CommandType. This can be file names, the file catalogue, etc.
    text_data: Option<String>,
    server_response: ServerResponse,
    current_location: Location,
}

impl Share {
    pub fn new(command: Command, current_location: Location) -> Share {
        Share { 
            command, 
            file: None,
            text_data: None, 
            server_response: ServerResponse::new(),
            current_location
        }
    }
    pub fn write_to_stream(&mut self, stream: &mut TcpStream, current_location: Location) -> Result<(), Box<dyn std::error::Error>>{
        let share = bincode::serialize(self)?;

        // Calculate the size (in bytes) of the struct
        let content_len = mem::size_of_val(&share[..]);

        // Send a header containing the content length and a newline
        stream.write(
        format!("{}\n",
                content_len
            ).as_bytes()
        )?;

        stream.write_all(&share[..])?;

        self.current_location = current_location;

        Ok(())
    }
    pub fn read_from_stream(stream: &mut TcpStream, current_location: Location) -> Result<Share, Box<dyn std::error::Error>> {
        let mut share_len: String = String::new() ;
        let mut buf_reader = BufReader::new(stream);
    
        // Read header
        buf_reader.read_line(&mut share_len)?;

        let share_len: usize = share_len.trim().parse()?;

        let mut share_bytes = Vec::new();
        share_bytes.resize(share_len, 0);
        
        buf_reader.read_exact(&mut share_bytes)?;

        let mut share = bincode::deserialize::<Share>(&share_bytes[..])?;

        share.current_location = current_location;

        Ok(share)
    }
    pub fn prepare_data(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        match *self.command.command_type() {
            CommandType::Exit => {
                process::exit(0);
            }
            CommandType::Help => {
                println!(
                    "{}\n{}\n{}\n{}\n{}",
                    "----- Help Guide -----",
                    "EXIT - Exit the client",
                    "UPLOAD [file] - Upload a file to the server",
                    "RECIEVE [file] - Recieve a file from the server",
                    "CATALOGUE - Recieve a list of files from the server",
                );
            }
            // Load file into vector
            CommandType::Upload if self.current_location == Location::Client => {
                let mut file = File::open(self.command.arg.as_ref().unwrap())?;

                self.file = Some(Vec::new());
                file.read_to_end(&mut self.file.as_mut().unwrap())?;
            },  

            _ => eprintln!("Nothing to prepare"),
        }

        Ok(())
    }
    pub fn execute(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if !self.command.command_type().is_client() {
            println!("Server says: {:?}. STATUS: {:?}", self.server_response.text, self.server_response.status);
        }

        if self.server_response.status == ServerResponseStatus::Error {
            return Ok(());
        }

        match *self.command.command_type() {
            // Recieved a file from the server; Move file inside memory to storage
            CommandType::Recieve if self.current_location == Location::Client => {
                let mut file = File::create(self.command.arg.as_ref().unwrap())?;

                file.write_all(&self.file.as_ref().unwrap())?;
            }
            // Send a file to the client; Move file inside storage to memory
            CommandType::Recieve if self.current_location == Location::Server => {
                let mut file = File::open(self.command.arg.as_ref().unwrap())?;

                self.file = Some(Vec::new());
                file.read_to_end(&mut self.file.as_mut().unwrap())?;
            }
            // Send a file to the client; Move file inside storage to memory
            CommandType::Upload if self.current_location == Location::Server => {
                let mut file = File::create(self.command.arg.as_ref().unwrap())?;

                file.write_all(&self.file.as_ref().unwrap())?;
            }
            // Print text_data containing a list of files the server has
            CommandType::Catalogue if self.current_location == Location::Client => {
                println!("{}", self.text_data.as_ref().unwrap());
            }
            // Load text_data with a list of files the server has
            CommandType::Catalogue if self.current_location == Location::Server => {
                let paths = fs::read_dir(".")?;

                for path in paths {
                    self.text_data = Some(String::new());

                    self.text_data.as_mut()
                        .unwrap()
                        .push_str(
                            format!("{}\n", path?.path().display()).as_str().clone()
                        );
                }
            }

            _ => (),
        }

        Ok(())
    }
    pub fn set_error_response(&mut self, error: Box<dyn std::error::Error>) {
        self.server_response.status = ServerResponseStatus::Error;
        self.server_response.text = Some(error.to_string());
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
enum ServerResponseStatus {
    Error,
    Success,
    None,
}

#[derive(Serialize, Deserialize, Debug)]
struct ServerResponse {
    status: ServerResponseStatus,

    text: Option<String>,
}

impl ServerResponse {
    fn new() -> ServerResponse {
        ServerResponse {
            status: ServerResponseStatus::Success,
            text: None,
        }
    }
}