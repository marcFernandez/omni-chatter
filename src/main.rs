use std::{
    sync::{Arc, Mutex},
    thread,
};

use anyhow::Result;
use tokio::sync::mpsc::unbounded_channel;

use crate::{command::CommandHandler, gui::run, messages::PlatformMessage};

mod command;
mod gui;
mod messages;
mod twitch;
mod youtube;

fn main() -> Result<()> {
    println!("ttv-bot");

    let command_handler = CommandHandler::new();

    let (sender, receiver) = unbounded_channel::<PlatformMessage>();

    let command_handler = Arc::new(Mutex::new(command_handler));
    let twitch_command_handler = command_handler.clone();
    let twitch_sender = sender.clone();
    let youtube_sender = sender.clone();

    let _twitch_thread = thread::spawn(|| twitch::listen(twitch_command_handler, twitch_sender));
    let _youtube_thread = thread::spawn(|| youtube::listen(youtube_sender));

    let gui_command_handler = command_handler.clone();

    let _ = run(gui_command_handler, receiver);

    Ok(())

    //loop {}
}
