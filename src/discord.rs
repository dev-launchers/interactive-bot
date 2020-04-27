use super::post::{post, PostError, Request};

use std::collections::HashMap;
use wasm_bindgen::JsValue;
use wasm_bindgen_futures::JsFuture;
use web_sys;

#[derive(Deserialize)]
pub struct DiscordConfig {
    // Shared secret to verify requests are from our discord-gateway
    pub gateway_token: String,
    pub webhook_url: String,
}

#[derive(Deserialize, Debug)]
struct Submission {
    submission: String,
    ts: u64,
}

pub async fn submit(req: web_sys::Request) -> Result<(), JsValue> {
    let body = JsFuture::from(req.json()?).await?;
    let submission: Submission = body
        .into_serde()
        .map_err(|e| format!("Failed to deserialize into Submission, err: {:?}", e))?;
    Ok(())
}

pub async fn checkLastSubmission(req: web_sys::Request) -> Result<(), JsValue> {
    Ok(())
}

#[derive(Serialize, Debug)]
struct WebhookBody {
    content: String,
}

pub struct WebhookClient {
    url: String,
}

pub fn new_webhook_client(url: String) -> WebhookClient {
    return WebhookClient { url };
}

impl WebhookClient {
    pub async fn execute(&self, message: String) -> Result<(), PostError> {
        let mut headers = HashMap::new();
        headers.insert("Content-type".to_string(), "application/json".to_string());
        let req = Request {
            url: self.url.clone(),
            headers: headers,
            body: WebhookBody { content: message },
        };
        // webhook doesn't return a response https://discordapp.com/developers/docs/resources/webhook#execute-webhook
        post(req).await?;
        Ok(())
    }
}
