use std::{
    sync::{Arc, Mutex},
    thread,
};

use anyhow::Result;

use crate::{command::CommandHandler, twitch::listen};

mod command;
mod twitch;

fn main() -> Result<()> {
    println!("ttv-bot");

    let command_handler = CommandHandler::new();

    let command_handler = Arc::new(Mutex::new(command_handler));
    let twitch_command_handler = command_handler.clone();
    thread::spawn(|| listen(twitch_command_handler));

    loop {}
}
