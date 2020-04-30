use super::emoji::LotteryConfig;
use super::http::{send, Method, PostError, Request};
use super::kv::{Guess, KVClient, WriteResponse};
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
    let client = KVClient::new(config.kv);
    let last_try = client
        .read(&submitter)
        .await
        .map_err(|e| format!("Can't retrieve last try"))?;

    let current_guess = Guess {
        value: submission.submission,
        created_at: submission.ts,
    };

    let retry_in_hrs = config.emoji.retry_in_hrs;
    match can_submit(&last_try, &current_guess, config.emoji) {
        Ok(_) => {
            let resp = client
                .write(submitter, current_guess)
                .await
                .map_err(|e| format!("Failed to submit, err: {:?}", e))?;

            match resp {
                WriteResponse::Ok(_) => Ok(JsValue::from_str(&format!(
                    "Not quite what I had in mind, try again in {:} hrs",
                    retry_in_hrs,
                ))),
                WriteResponse::Err(e) => Err(JsValue::from_str(&format!(
                    "Failed to submit, err: {:?}",
                    e,
                ))),
            }
        }
        Err(e) => Ok(JsValue::from_str(&format!("please submit after {:?}", e))),
    }
}

fn can_submit(
    last_try: &Guess,
    current_guess: &Guess,
    emoji_config: LotteryConfig,
) -> Result<(), DateTime<Utc>> {
    let last_time = NaiveDateTime::from_timestamp(last_try.created_at, 0);
    let current_time = NaiveDateTime::from_timestamp(current_guess.created_at, 0);
    let next_retryable_time = last_time + chrono::Duration::hours(emoji_config.retry_in_hrs);
    if current_time > next_retryable_time {
        return Ok(());
    }
    Err(DateTime::from_utc(next_retryable_time, Utc))
}

pub async fn checkLastSubmission(submitter: String, config: BotConfig) -> Result<JsValue, JsValue> {
    let client = KVClient::new(config.kv);
    let resp = client
        .read(&submitter)
        .await
        .map_err(|e| format!("Failed to check last submission, err: {:?}", e))?;
    Ok(JsValue::from_str(&resp.to_string()))
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
            method: Method::POST,
            headers: headers,
            body: WebhookBody { content: message },
        };
        // webhook doesn't return a response https://discordapp.com/developers/docs/resources/webhook#execute-webhook
        send(req).await?;
        Ok(())
    }
}
