pub enum Platform {
    Twitch,
    Youtube,
}

pub struct PlatformMessage {
    pub sender: String,
    pub msg: String,
    pub platform: Platform,
}
