use std::{
    fmt,
    fs::{File, OpenOptions},
    io::{Read, Write},
};

use serde::{Deserialize, Serialize};
use serde_json::{from_str, Value};

type HandleCommandResult<T> = std::result::Result<T, MissingCommand>;

pub static COMMAND_SYMBOL: char = '!';

#[derive(Debug, Clone)]
pub struct MissingCommand {
    name: String,
}

impl fmt::Display for MissingCommand {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Invalid command: {}", self.name)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BotCommand {
    name: String,
    contents: String,
}

static COMMANDS_FILE: &str = "commands.json";

fn is_sender_allowed(sender: &str) -> bool {
    return sender.eq("zartisimo");
}

pub struct CommandHandler {
    contents: Value,
}

fn read_file() -> Value {
    let mut file = File::open(COMMANDS_FILE).expect("Commands file to exist");
    let mut contents = String::new();
    let _bytes = file.read_to_string(&mut contents).expect("The file to be UTF-8");
    return from_str(&contents).expect("The file to be a valid JSON");
}

impl CommandHandler {
    pub fn new() -> Self {
        let contents = read_file();
        Self { contents }
    }

    fn write_file(&self) {
        let mut file = OpenOptions::new()
            .truncate(true)
            .write(true)
            .create(true)
            .open(COMMANDS_FILE)
            .expect("To open or create commands file");
        match file.write_all(self.contents.to_string().as_bytes()) {
            Err(err) => eprintln!("WARN - Cannot write commands file: {:?}", err),
            _ => {}
        };
    }

    pub fn handle_command(&mut self, sender: String, msg: String) -> HandleCommandResult<Option<String>> {
        // !today
        // !settoday args
        let mut iter = msg.split(" ");
        let command = iter.next().expect("Message to not be empty");
        if command.starts_with("set") {
            return self.handle_set_command(sender, command[3..].to_string(), iter.collect::<Vec<&str>>().join(" "));
        }

        match self.contents.get(command) {
            Some(command) => Ok(Some(command["contents"].to_string())),
            None => Err(MissingCommand { name: command.to_string() }),
        }
    }

    /// This method expects the msg to be `command <args>`, where <args> represents the rest of the
    /// message. It will update the command contents and save it to the commands file.
    ///
    /// # Errors
    /// Returns `MissingCommand` if the command is not found.
    ///
    /// # Example
    /// ```
    /// self.handle_set_command("today", "Today we will build bla bla bla".to_string())
    /// ```
    fn handle_set_command(&mut self, sender: String, command_name: String, new_contents: String) -> HandleCommandResult<Option<String>> {
        if !is_sender_allowed(&sender) {
            return Ok(Some(format!("I'm sorry {}. You are not allowed to execute this command.", sender)));
        }

        match self.contents.get_mut(&command_name) {
            Some(contents) => {
                contents["contents"] = Value::String(new_contents);
                self.write_file();
                Ok(Some(format!("Updated {} command", command_name)))
            }
            None => Err(MissingCommand { name: command_name }),
        }
    }
}
