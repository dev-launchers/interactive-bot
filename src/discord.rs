use super::error::Error;
use super::http::{send, Method, Request};
use super::kv::{Guess, KVClient};
use super::BotConfig;

use chrono::prelude::*;
use std::collections::HashMap;
use wasm_bindgen::JsValue;
use wasm_bindgen_futures::JsFuture;
use web_sys;

#[derive(Deserialize)]
pub struct DiscordConfig {
    // Shared secret to verify requests are from our discord-gateway
    pub gateway_token: String,
    pub webhook_url: String,
    pub maintainer: String,
}

#[derive(Deserialize, Debug)]
struct Submission {
    submission: String,
    ts: i64,
}

pub async fn submit(
    req: web_sys::Request,
    submitter: String,
    config: BotConfig,
) -> Result<JsValue, JsValue> {
    if !config.emoji.active {
        return Ok(JsValue::from_str(
            "No active lottery yet, wait for the next announcement!",
        ));
    }
    let body = JsFuture::from(req.json()?).await?;
    let submission: Submission = body
        .into_serde()
        .map_err(|e| format!("Failed to deserialize into Submission, err: {:?}", e))?;

    if submission.submission == config.emoji.jackpot {
        return Ok(JsValue::from_str("Bingo!"));
    }
    let client = KVClient::new(config.kv, config.emoji.data_kv_namespace);
    let last_try = client
        .read(&submitter)
        .await
        .map_err(|e| format!("Can't retrieve last try, err: {:?}", e))?;

    let current_guess = Guess {
        value: submission.submission,
        created_at: submission.ts,
    };

    let retry_in_hrs = config.emoji.retry_in_hrs;
    match last_try {
        Some(l) => {
            let last_time = NaiveDateTime::from_timestamp(l.created_at, 0);
            let current_time = NaiveDateTime::from_timestamp(current_guess.created_at, 0);
            let next_retryable_time = last_time + chrono::Duration::hours(retry_in_hrs);
            if current_time < next_retryable_time {
                return Ok(JsValue::from_str(&format!(
                    "please submit after {:?}",
                    next_retryable_time
                )));
            }
        }
        None => {}
    };

    let resp = client
        .write::<Guess>(&submitter, current_guess)
        .await
        .map_err(|e| format!("Failed to submit, err: {:?}", e))?;

    Ok(JsValue::from_str(&format!(
        "Not quite what I had in mind, try again in {:} hrs",
        retry_in_hrs,
    )))
}

pub async fn checkLastSubmission(submitter: String, config: BotConfig) -> Result<JsValue, JsValue> {
    let client = KVClient::new(config.kv, config.emoji.data_kv_namespace);
    let resp = client
        .read(&submitter)
        .await
        .map_err(|e| format!("Failed to check last submission, err: {:?}", e))?;
    match resp {
        Some(s) => Ok(JsValue::from_str(&s.to_string())),
        None => Ok(JsValue::from_str("You haven't submit anything yet!")),
    }
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
    pub async fn execute(&self, message: String) -> Result<(), Error> {
        let mut headers = HashMap::new();
        headers.insert("Content-type".to_string(), "application/json".to_string());
        let req = Request {
            url: self.url.clone(),
            method: Method::POST,
            headers: headers,
            body: WebhookBody { content: message },
        };
        // webhook doesn't return a response https://discordapp.com/developers/docs/resources/webhook#execute-webhook
        send(req).await?;
        Ok(())
    }
}
