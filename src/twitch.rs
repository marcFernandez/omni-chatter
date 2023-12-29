use std::{
    env,
    net::TcpStream,
    sync::{Arc, Mutex},
};

use anyhow::Result;
use tungstenite::{connect, stream::MaybeTlsStream, Message, WebSocket};

use crate::command::{CommandHandler, COMMAND_SYMBOL};

#[derive(Debug, Clone)]
pub enum TwitchMessage {
    JOIN { channel: String },
    PRIVMSG { sender: String, channel: String, msg: String },
    UNIMPLEMENTED { msg: String },
}

impl Into<TwitchMessage> for &str {
    fn into(self) -> TwitchMessage {
        //:zartisimo!zartisimo@zartisimo.tmi.twitch.tv PRIVMSG #zartisimo :test
        //|__________________________________________| |_____| |________| |____...
        let mut iter = self.split(" ");
        let sender: String = iter.next().expect("sender part to be present in twitch message")[1..]
            .split("!")
            .next()
            .expect("sender to be present in twitch message sender part")
            .into();
        let command = iter.next().expect("command to be present in twitch message");
        match command {
            "PRIVMSG" => {
                let channel: String = iter.next().expect("channel to be present")[1..].into();
                let mut msg = iter.collect::<Vec<&str>>().join(" ")[1..].to_string();
                msg.remove(msg.len() - 1);
                msg.remove(msg.len() - 1);
                TwitchMessage::PRIVMSG { sender, channel, msg }
            }
            _ => TwitchMessage::UNIMPLEMENTED { msg: self.to_string() },
        }
    }
}

impl Into<String> for TwitchMessage {
    fn into(self) -> String {
        match self {
            TwitchMessage::JOIN { channel } => {
                format!("JOIN #{channel}")
            }
            TwitchMessage::PRIVMSG { sender: _, channel, msg } => {
                format!("PRIVMSG #{channel} :{msg}")
            }
            TwitchMessage::UNIMPLEMENTED { msg: _ } => todo!("Probably won't exist"),
        }
    }
}

pub fn get_twitch_stream() -> Result<WebSocket<MaybeTlsStream<TcpStream>>> {
    let token = env::var("TWITCH_TOKEN").expect("TWITCH_TOKEN to be defined");

    let (mut stream, _) = connect("ws://irc-ws.chat.twitch.tv:80").expect("to connect");

    stream.send(Message::Text(format!("PASS oauth:{token}")))?;
    stream.send(Message::Text(format!("NICK zartisimo")))?;
    stream.send(Message::Text(TwitchMessage::JOIN { channel: "zartisimo".to_string() }.into()))?;
    return Ok(stream);
}

#[allow(dead_code)]
pub fn get_twitch_stream_arcmutex() -> Result<Arc<Mutex<WebSocket<MaybeTlsStream<TcpStream>>>>> {
    return Ok(Arc::new(Mutex::new(get_twitch_stream()?)));
}

// TODO: Maybe create a command queue in case the CommandHandler is busy and cannot be locked
// (this is kinda too optimistic for a stream with 1 viewer XD)
pub fn listen(command_handler: Arc<Mutex<CommandHandler>>) -> Result<()> {
    let mut stream = get_twitch_stream()?;

    loop {
        let msg = stream
            .read()
            .map_err(|err| {
                eprintln!("Error reading msg from stream: {:?}", err);
            })
            .expect("msg");

        let msg: TwitchMessage = msg.to_text().unwrap().into();

        match msg {
            TwitchMessage::PRIVMSG { sender, channel: _, msg } => {
                if msg.starts_with(COMMAND_SYMBOL) {
                    let response = command_handler
                        .lock()
                        .expect("To lock command_handler for Twitch thread")
                        // TODO: find the best way to do this
                        .handle_command(sender, msg[1..].to_string())
                        .unwrap_or(None);
                    if let Some(response) = response {
                        stream.send(Message::Text(
                            TwitchMessage::PRIVMSG {
                                sender: "zartisimo".to_string(),
                                channel: "zartisimo".to_string(),
                                msg: response,
                            }
                            .into(),
                        ))?;
                    }
                }
            }
            TwitchMessage::JOIN { channel: _ } => {}
            TwitchMessage::UNIMPLEMENTED { msg: _ } => {}
        }
    }
}
