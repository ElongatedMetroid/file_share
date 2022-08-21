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

        let command = Command { command_type, arg };

        Ok(command)
    }
    
}