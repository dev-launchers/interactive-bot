use super::post::{post, PostError, Request};
use std::collections::HashMap;
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
    as_user: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PostMessageResp {
    pub ok: bool,
    pub error: Option<String>,
    #[serde(rename(deserialize = "ts"))]
    timestamp: Option<String>,
}

pub async fn post_message(channel: String, token: String) -> Result<PostMessageResp, PostError> {
    let mut headers = HashMap::new();
    headers.insert("Authorization".to_string(), format!("Bearer {}", token));
    headers.insert("Content-type".to_string(), "application/json".to_string());
    let req = Request {
        url: "https://slack.com/api/chat.postMessage".to_string(),
        headers: headers,
        body: PostMessageBody {
            channel: channel,
            text: "Hello, I'm lottery bot".to_string(),
            as_user: None,
        },
    };
    let resp = post(req).await?;
    Ok(resp)
}
