use std::{
    collections::HashSet,
    fmt,
    fs::{File, OpenOptions},
    io::{Read, Write},
};

use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::{from_str, from_value, Map, Value};

type HandleCommandResult<T> = std::result::Result<T, HandleCommandError>;

pub static COMMAND_SYMBOL: char = '!';
pub static CREATE_COMMAND_SYMBOL: char = '#';

#[derive(Debug)]
pub enum HandleCommandError {
    MissingCommand(String),
    CreateCommand(CreateCommandError),
}

impl fmt::Display for HandleCommandError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            HandleCommandError::MissingCommand(name) => {
                write!(f, "Invalid command: {}", name)
            }
            HandleCommandError::CreateCommand(err) => {
                write!(f, "{}", err)
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct CreateCommandError {
    name: String,
    msg: String,
}

impl fmt::Display for CreateCommandError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Error creating command {}: {}", self.name, self.msg)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BotCommand {
    pub name: String,
    pub contents: String,
}

impl Into<BotCommand> for &Map<String, Value> {
    fn into(self) -> BotCommand {
        BotCommand {
            name: self
                .get("name")
                .expect("Command to have name property")
                .as_str()
                .unwrap()
                .to_string(),
            contents: self
                .get("contents")
                .expect("Command to have name property")
                .as_str()
                .unwrap()
                .to_string(),
        }
    }
}

static COMMANDS_FILE: &str = "commands.json";

fn is_sender_allowed(sender: &str) -> bool {
    return sender.eq("zartisimo");
}

pub struct CommandHandler {
    contents: Value,
    command_names: HashSet<String>,
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
        let commands = contents.as_object().unwrap();
        let command_names = commands.keys().map(|key| key.to_string()).collect();
        Self {
            contents,
            command_names,
        }
    }

    pub fn get_command(&self, command_name: &String) -> BotCommand {
        from_value(self.contents.get(command_name).expect("Command to exist").to_owned())
            .expect("Command in file to be valid BotCommand")
        // For now let's assume the command always exists
        //from_str(self.contents.get(command_name).unwrap().as_object().unwrap()).expect("Command from file to be valid BotCommand")
    }

    pub fn get_command_names(&mut self) -> HashSet<String> {
        if !self.command_names.contains("help") {
            self.generate_help_command();
        }
        self.command_names.clone()
    }

    pub fn generate_help_command(&mut self) {
        let mut help_contents = String::from("Available commands:");
        for command_name in &self.command_names {
            help_contents = format!("{help_contents} {COMMAND_SYMBOL}{command_name}");
        }

        let help_command = BotCommand {
            name: "help".to_string(),
            contents: help_contents,
        };

        self.create_command(&help_command).expect("To create the help command");
        self.command_names.insert(help_command.name);
        self.write_file();
    }

    pub fn update_command_content(&mut self, command_name: &String, new_contents: &String) -> Result<()> {
        match self.contents.get_mut(&command_name) {
            Some(contents) => {
                contents["contents"] = Value::String(new_contents.to_string());
                self.write_file();
                Ok(())
            }
            None => Err(anyhow::Error::msg("Command does not exist")),
        }
    }

    pub fn delete_command(&mut self, command_name: &String) -> Result<()> {
        if !self.command_names.contains(command_name) {
            return Err(anyhow::Error::msg(format!("Command {} does not exist", command_name)));
        }

        let mut new_contents: Map<String, Value> = Map::new();

        for name in &self.command_names {
            if *name != *command_name {
                let command = &self.get_command(&name);
                new_contents.insert(
                    name.to_string(), // can this be achieved without copying?
                    serde_json::from_str(
                        serde_json::to_string(&command)
                            .expect("command to be valid BotCommand")
                            .as_str(),
                    )
                    .expect("stringified BotCommand to be valid Json"),
                );
            }
        }

        self.contents = Value::Object(new_contents);
        self.command_names.remove(command_name);
        self.write_file();

        Ok(())
    }

    pub fn create_command(&mut self, command: &BotCommand) -> Result<()> {
        // TODO: handle whitespace-only name and contents
        if command.name.is_empty() {
            return Err(anyhow::Error::msg("Command name cannot be empty"));
        }

        if command.name.find(' ').is_some() {
            return Err(anyhow::Error::msg("Command name cannot contain spaces"));
        }

        if command.contents.is_empty() {
            return Err(anyhow::Error::msg("Command contents cannot be empty"));
        }

        if self.command_names.contains(&command.name) {
            return Err(anyhow::Error::msg(format!("Command {} alredy exists", command.name)));
        }

        self.contents[&command.name] = serde_json::from_str(
            serde_json::to_string(command)
                .expect("command to be valid BotCommand")
                .as_str(),
        )
        .expect("stringified BotCommand to be valid Json");
        self.command_names.insert(command.name.clone());

        self.write_file();

        Ok(())
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

        if command.starts_with(CREATE_COMMAND_SYMBOL) {
            return self.handle_create_command(sender, command[1..].to_string(), iter.collect::<Vec<&str>>().join(" "));
        }

        match self.contents.get(command) {
            Some(command) => Ok(Some(command["contents"].as_str().unwrap().to_string())),
            None => Err(HandleCommandError::MissingCommand(command.to_string())),
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
    fn handle_set_command(
        &mut self,
        sender: String,
        command_name: String,
        new_contents: String,
    ) -> HandleCommandResult<Option<String>> {
        if !is_sender_allowed(&sender) {
            return Ok(Some(format!(
                "I'm sorry {}. You are not allowed to execute this command.",
                sender
            )));
        }

        match self.contents.get_mut(&command_name) {
            Some(contents) => {
                contents["contents"] = Value::String(new_contents);
                self.write_file();
                Ok(Some(format!("Updated {} command", command_name)))
            }
            None => Err(HandleCommandError::MissingCommand(command_name.to_string())),
        }
    }

    fn handle_create_command(
        &mut self,
        sender: String,
        command_name: String,
        new_contents: String,
    ) -> Result<Option<String>, HandleCommandError> {
        if !is_sender_allowed(&sender) {
            return Ok(Some(format!(
                "I'm sorry {}. You are not allowed to execute this command.",
                sender
            )));
        }

        if self.contents.get(&command_name).is_some() {
            let msg = format!("ERROR - Command {command_name} already exists");
            return Err(HandleCommandError::CreateCommand(CreateCommandError {
                name: command_name,
                msg,
            }));
        }

        if new_contents.eq("") {
            let msg = format!("ERROR - No content for new command {command_name}");
            return Err(HandleCommandError::CreateCommand(CreateCommandError {
                name: command_name,
                msg,
            }));
        }

        let new_command = BotCommand {
            name: command_name,
            contents: new_contents,
        };

        self.create_command(&new_command)
            .expect("To be able to create the command");
        self.write_file();

        Ok(Some(format!("Created {} command", new_command.name)))
    }
}
