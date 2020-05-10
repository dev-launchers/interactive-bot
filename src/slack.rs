use super::error::Error;
use super::http::{send, Method, Request};
use std::collections::HashMap;
use wasm_bindgen_futures::JsFuture;

#[derive(Deserialize)]
pub struct SlackConfig {
    token: String,
    announcement_channel: String,
    pub maintainer: String,
}

/*
https://api.slack.com/events/message
Sample MessageEvent
{
    "type": "message",
    "channel": "C2147483705",
    "user": "U2147483697",
    "text": "Hello world",
    "ts": "1355517523.000005"
}
*/
#[derive(Deserialize, Debug)]
pub struct MessageEvent {
    /*
    error: expected identifier, found keyword `type`
      --> src/lib.rs:50:5
       |
    50 |     type: String,
       |     ^^^^ expected identifier, found keyword
       |
    help: you can escape reserved keywords to use them as identifiers
       |
    50 |     r#type: String,
       |     ^^^^^^
    */
    r#type: String,
    channel: String,
    user: String,
    text: String,
    // https://github.com/slackhq/slack-api-docs/issues/7
    // Slack message timestamps should be stored and compared as strings
    #[serde(rename(deserialize = "ts"))]
    timestamp: String,
}

// https://api.slack.com/methods/chat.postMessage
#[derive(Serialize, Debug)]
struct PostMessageBody {
    channel: String,
    text: String,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum PostMessageResp {
    Ok(PostMessageOk),
    Err(PostMessageError),
}

#[derive(Debug, Deserialize)]
pub struct PostMessageOk {
    ok: bool,
    channel: String,
    ts: String,
}

#[derive(Debug, Deserialize)]
pub struct PostMessageError {
    ok: bool,
    pub error: String,
}

pub struct SlackClient {
    config: SlackConfig,
}

pub fn new_slack_client(config: SlackConfig) -> SlackClient {
    return SlackClient { config };
}

impl SlackClient {
    pub async fn post_message(&self, message: String) -> Result<PostMessageResp, Error> {
        let mut headers = HashMap::new();
        headers.insert(
            "Authorization".to_string(),
            format!("Bearer {}", self.config.token),
        );
        headers.insert("Content-type".to_string(), "application/json".to_string());
        let req = Request {
            url: "https://slack.com/api/chat.postMessage".to_string(),
            method: Method::POST,
            headers: headers,
            body: PostMessageBody {
                channel: self.config.announcement_channel.clone(),
                text: message,
            },
        };
        let js_resp = send(req).await?;

        // Convert this Promise into a rust Future.
        let json = JsFuture::from(js_resp.json()?).await?;

        let resp: PostMessageResp = json.into_serde()?;

        Ok(resp)
    }
}
