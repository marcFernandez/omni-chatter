use std::{
    sync::{Arc, Mutex},
    thread,
};

use anyhow::Result;

use crate::{command::CommandHandler, twitch::listen, gui::run};

mod command;
mod twitch;
mod gui;

fn main() -> Result<()> {
    println!("ttv-bot");

    let command_handler = CommandHandler::new();

    let command_handler = Arc::new(Mutex::new(command_handler));
    let twitch_command_handler = command_handler.clone();
    thread::spawn(|| listen(twitch_command_handler));

    let gui_command_handler = command_handler.clone();

    let _ = run(gui_command_handler);
    Ok(())

    //loop {}
}
