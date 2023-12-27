use std::{
    env,
    net::TcpStream,
    sync::{Arc, Mutex},
};

use anyhow::Result;
use tungstenite::{connect, stream::MaybeTlsStream, Message, WebSocket};

fn is_sender_allowed(sender: &str) -> bool {
    return sender.eq("zartisimo");
}

#[derive(Debug)]
struct BotCommand {
    sender: String,
    #[allow(dead_code)]
    command: BotCommands,
    #[allow(dead_code)]
    is_public: bool,
    args: String,
    run: fn(String, String, Arc<Mutex<WebSocket<MaybeTlsStream<TcpStream>>>>),
}

impl BotCommand {
    pub fn run(self, stream: Arc<Mutex<WebSocket<MaybeTlsStream<TcpStream>>>>) {
        if !is_sender_allowed(&self.sender) {
            return;
        }

        (self.run)(self.sender, self.args, stream);
    }
}

fn today(sender: String, _args: String, stream: Arc<Mutex<WebSocket<MaybeTlsStream<TcpStream>>>>) {
    if let Ok(mut stream) = stream.try_lock() {
        let msg = format!("Hi {sender}! Today we are building a bot for my twitch channel");
        let _ = stream.send(Message::Text(
            TwitchMessage::PRIVMSG {
                sender: String::from("zartisimo"),
                channel: String::from("zartisimo"),
                msg,
            }
            .into(),
        ));
    }
}

fn set_today(_: String, args: String, stream: Arc<Mutex<WebSocket<MaybeTlsStream<TcpStream>>>>) {
    if let Ok(mut stream) = stream.try_lock() {
        let msg = format!("updated today to '{}'", args);
        let _ = stream.send(Message::Text(
            TwitchMessage::PRIVMSG {
                sender: String::from("zartisimo"),
                channel: String::from("zartisimo"),
                msg,
            }
            .into(),
        ));
    }
}

fn socials(_sender: String, _args: String, stream: Arc<Mutex<WebSocket<MaybeTlsStream<TcpStream>>>>) {
    if let Ok(mut stream) = stream.try_lock() {
        let msg = String::from("Github: https://github.com/marcFernandez");
        let _ = stream.send(Message::Text(
            TwitchMessage::PRIVMSG {
                sender: String::from("zartisimo"),
                channel: String::from("zartisimo"),
                msg,
            }
            .into(),
        ));
    }
}

impl Into<Option<BotCommand>> for TwitchMessage {
    fn into(self) -> Option<BotCommand> {
        match self {
            TwitchMessage::PRIVMSG { sender, channel: _, msg } => {
                // Expect commands to be <symbol><name> <args>
                let mut iter = msg.split(" ");
                let command = iter.next().expect("Message to not be empty");
                match command {
                    "!today" => Some(BotCommand {
                        sender,
                        command: BotCommands::TODAY,
                        is_public: true,
                        args: String::new(),
                        run: today,
                    }),
                    "!setToday" => Some(BotCommand {
                        sender,
                        command: BotCommands::SetToday,
                        is_public: false,
                        args: iter.collect::<Vec<&str>>().join(" "),
                        run: set_today,
                    }),
                    "!socials" => Some(BotCommand {
                        sender,
                        command: BotCommands::SOCIALS,
                        is_public: false,
                        args: String::new(),
                        run: socials,
                    }),
                    _ => None,
                }
            }
            _ => None,
        }
    }
}

#[derive(Debug, PartialEq)]
enum BotCommands {
    TODAY,
    SetToday,
    SOCIALS,
}

#[derive(Debug, Clone)]
enum TwitchMessage {
    JOIN {
        channel: String,
    },
    PRIVMSG {
        sender: String,
        channel: String,
        msg: String,
    },
    #[allow(dead_code)]
    UNIMPLEMENTED {
        msg: String,
    },
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

fn main() -> Result<()> {
    println!("ttv-bot");
    let token = env::var("TWITCH_TOKEN").expect("TWITCH_TOKEN to be defined");

    let (stream, _) = connect("ws://irc-ws.chat.twitch.tv:80").expect("to connect");
    let stream = Arc::new(Mutex::new(stream));

    stream.lock().unwrap().send(Message::Text(format!("PASS oauth:{token}")))?;
    stream.lock().unwrap().send(Message::Text(format!("NICK zartisimo")))?;
    stream
        .lock()
        .unwrap()
        .send(Message::Text(TwitchMessage::JOIN { channel: "zartisimo".to_string() }.into()))?;

    loop {
        let msg = stream
            .lock()
            .unwrap()
            .read()
            .map_err(|err| {
                eprintln!("Error reading msg from stream: {:?}", err);
            })
            .expect("msg");

        let msg: TwitchMessage = msg.to_text().unwrap().into();
        let bot_command: Option<BotCommand> = msg.clone().into();
        match bot_command {
            Some(bot_command) => bot_command.run(stream.clone()),
            None => {}
        }
    }
}
