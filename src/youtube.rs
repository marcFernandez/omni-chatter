use std::{
    thread,
    time::Duration,
};

use tokio::sync::mpsc::UnboundedSender;

use crate::messages::{Platform, PlatformMessage};

pub fn listen(sender: UnboundedSender<PlatformMessage>) {
    loop {
        sender
            .send(PlatformMessage {
                sender: "yt-username".to_string(),
                msg: "Hi this is a yt message".to_string(),
                platform: Platform::Youtube,
            })
            .expect("To send the PlatformMessage");

        thread::sleep(Duration::from_secs(10));
    }
}
