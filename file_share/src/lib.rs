#![feature(core_intrinsics)]

use std::{process, fs::File, io::{Read, Write, BufReader, BufRead}, mem, net::TcpStream};

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
    pub fn execute_client_side(&self) -> Result<(), &'static str> {
        match *self.command_type() {
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
            _ => return Err("Tried to execute serverside command on client"),
        }

        Ok(())
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Share {
    command: Command,

    /// Contains file data
    file: Option<Vec<u8>>,
    /// Contains text data, this is interpretted diferent ways depending on the
    /// CommandType. This can be file names, the file catalogue, etc.
    text_data: Option<String>,
    /// Is None when no error has occured
    server_error_response: Option<String>,
}

impl Share {
    pub fn new(command: Command) -> Share {
        Share { 
            command, 
            file: None,
            text_data: None, 
            server_error_response: None 
        }
    }
    pub fn prepare_data(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        match *self.command.command_type() {
            // Load file into vector
            CommandType::Upload => {
                let mut file = File::open(self.command.arg.as_ref().unwrap())?;

                self.file = Some(Vec::new());
                file.read_to_end(&mut self.file.as_mut().unwrap())?;
            },  

            _ => eprintln!("Nothing to prepare"),
        }

        Ok(())
    }
    pub fn write_to_stream(&mut self, stream: &mut TcpStream) -> Result<(), Box<dyn std::error::Error>>{
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

        stream.flush()?;

        Ok(())
    }
    pub fn read_from_stream(mut stream: TcpStream) -> Result<Share, Box<dyn std::error::Error>> {
        let mut share_len: String = String::new() ;
        let mut buf_reader = BufReader::new(&mut stream);
    
        // Read header
        buf_reader.read_line(&mut share_len)?;

        let share_len: usize = share_len.trim().parse()?;

        let mut share_bytes = Vec::new();
        share_bytes.resize(share_len, 0);
        
        buf_reader.read_exact(&mut share_bytes)?;

        stream.flush()?;

        Ok(bincode::deserialize::<Share>(&share_bytes[..])?)
    }
}