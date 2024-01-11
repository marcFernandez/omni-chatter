use serde::Deserialize;
use serde::Serialize;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct YoutubeResponse {
    pub kind: String,
    pub etag: String,
    pub polling_interval_millis: i64,
    pub page_info: PageInfo,
    pub next_page_token: String,
    pub items: Vec<LiveChatMessage>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PageInfo {
    pub total_results: i64,
    pub results_per_page: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LiveChatMessage {
    pub kind: String,
    pub etag: String,
    pub id: String,
    pub snippet: Snippet,
    pub author_details: AuthorDetails,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Snippet {
    #[serde(rename = "type")]
    pub type_field: String,
    pub live_chat_id: String,
    pub author_channel_id: String,
    pub published_at: String,
    pub has_display_content: bool,
    pub display_message: String,
    pub text_message_details: TextMessageDetails,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TextMessageDetails {
    pub message_text: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthorDetails {
    pub channel_id: String,
    pub channel_url: String,
    pub display_name: String,
    pub profile_image_url: String,
    pub is_verified: bool,
    pub is_chat_owner: bool,
    pub is_chat_sponsor: bool,
    pub is_chat_moderator: bool,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LiveStreamsResponse {
    pub kind: String,
    pub etag: String,
    pub page_info: PageInfo,
    pub items: Vec<LiveStream>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LiveStream {
    pub kind: String,
    pub etag: String,
    pub id: String,
    pub snippet: LiveStreamSnippet,
    pub status: Status,
    pub content_details: ContentDetails,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LiveStreamSnippet {
    pub published_at: String,
    pub channel_id: String,
    pub title: String,
    pub description: String,
    pub thumbnails: Thumbnails,
    pub actual_start_time: String,
    pub is_default_broadcast: bool,
    pub live_chat_id: String,
    pub actual_end_time: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Thumbnails {
    pub default: Default,
    pub medium: Medium,
    pub high: High,
    pub standard: Standard,
    pub maxres: Maxres,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Default {
    pub url: String,
    pub width: i64,
    pub height: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Medium {
    pub url: String,
    pub width: i64,
    pub height: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct High {
    pub url: String,
    pub width: i64,
    pub height: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Standard {
    pub url: String,
    pub width: i64,
    pub height: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Maxres {
    pub url: String,
    pub width: i64,
    pub height: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Status {
    pub life_cycle_status: String,
    pub privacy_status: String,
    pub recording_status: String,
    pub made_for_kids: bool,
    pub self_declared_made_for_kids: bool,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContentDetails {
    pub bound_stream_id: String,
    pub bound_stream_last_update_time_ms: String,
    pub monitor_stream: MonitorStream,
    pub enable_embed: bool,
    pub enable_dvr: bool,
    pub enable_content_encryption: bool,
    pub start_with_slate: bool,
    pub record_from_start: bool,
    pub enable_closed_captions: bool,
    pub closed_captions_type: String,
    pub enable_low_latency: bool,
    pub latency_preference: String,
    pub projection: String,
    pub enable_auto_start: bool,
    pub enable_auto_stop: bool,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MonitorStream {
    pub enable_monitor_stream: bool,
    pub broadcast_stream_delay_ms: i64,
    pub embed_html: String,
}
