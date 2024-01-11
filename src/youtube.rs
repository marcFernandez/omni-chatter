use crate::{
    messages::{Platform, PlatformMessage},
    youtube_model::{LiveStreamsResponse, YoutubeResponse},
};
use anyhow::Result;
use reqwest::{blocking::Client, header::ACCEPT};
use std::{env, thread, time::Duration};
use tokio::sync::mpsc::UnboundedSender;

pub fn listen(sender: UnboundedSender<PlatformMessage>) -> Result<()> {
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
    // WIP
    /*
    let youtube_token = env::var("YOUTUBE_TOKEN").expect("YOUTUBE_TOKEN to be defined");
    let client = Client::new();
    // TODO: Find a way to get live_chat_id dynamically. Otherwise, add link to request it on
    // Youtube API docs: https://developers.google.com/youtube/v3/live/docs/liveBroadcasts/list
    let url = format!("https://youtube.googleapis.com/youtube/v3/liveBroadcasts?part=snippet%2CcontentDetails%2Cstatus&broadcastType=all&mine=true&key={}", youtube_token);
    let res = client
        .get(url)
        .header(ACCEPT, "application/json")
        .send()
        .map_err(|err| eprintln!("Error getting LiveStreams: {:?}", err))
        .unwrap();
    println!("res: {:?}", res);
    let res = res
        .json::<LiveStreamsResponse>()
        .map_err(|err| eprintln!("Error parsing LiveStreams: {:?}", err))
        .expect("LiveStreamsResponse");
    let live_chat_id = &res.items.first().expect("livestream").snippet.live_chat_id;
    let url = format!("https://youtube.googleapis.com/youtube/v3/liveChat/messages?liveChatId={}&part=id&part=snippet&part=authorDetails&key={}",
                      live_chat_id, youtube_token);
    let mut page_token = String::new();
    loop {
        let mut full_url = url.clone();
        if page_token != "" {
            full_url.push_str(&format!("&pageToken={}", page_token));
        }
        println!("Making request to Youtube");
        match client.get(full_url).header(ACCEPT, "application/json").send() {
            Ok(r) => {
                println!("Response: {:?}", r);
                match r.json::<YoutubeResponse>() {
                    Ok(response) => {
                        println!("Received {} yt messages", response.items.len());
                        // TODO: Persist this somewhere to not reload old messages on restart
                        page_token = response.next_page_token;
                        for msg in response.items {
                            match msg.snippet.type_field.as_str() {
                                "textMessageEvent" => {
                                    sender
                                        .send(PlatformMessage {
                                            sender: msg.author_details.display_name,
                                            msg: msg.snippet.text_message_details.message_text,
                                            platform: Platform::Youtube,
                                        })
                                        .expect("To send the PlatformMessage");
                                }
                                "superChatEvent" => {}
                                _ => {}
                            }
                        }
                    }
                    Err(err) => {
                        eprintln!("Error parsing response: {:?}", err);
                    }
                }
            }
            Err(err) => {
                eprintln!("Error sending request: {:?}", err);
            }
        }

        thread::sleep(Duration::from_secs(5));
    }
    */
}
