use super::http::{send, Method, PostError, Request};
use chrono::prelude::*;
use js_sys::ArrayBuffer;
use serde_json;
use std::collections::HashMap;
use std::fmt;
use wasm_bindgen_futures::JsFuture;
use web_sys::TextDecoder;

#[derive(Deserialize)]
pub struct KVConfig {
    token: String,
    account_id: String,
    namespace_id: String,
}

pub struct KVClient {
    config: KVConfig,
}

impl KVClient {
    pub fn new(config: KVConfig) -> KVClient {
        KVClient { config }
    }

    pub async fn read(&self, key: &str) -> Result<Guess, PostError> {
        let mut headers = HashMap::new();
        headers.insert(
            "Authorization".to_string(),
            format!("Bearer {}", self.config.token),
        );
        headers.insert("Content-type".to_string(), "application/json".to_string());
        let url = format!(
            "https://api.cloudflare.com/client/v4/accounts/{}/storage/kv/namespaces/{}/values/{}",
            self.config.account_id, self.config.namespace_id, key
        );
        let req = Request {
            url: url,
            method: Method::GET,
            headers: headers,
            body: (),
        };
        let js_resp = send(req).await?;

        // Convert this Promise into a rust Future.
        let js_value = JsFuture::from(js_resp.array_buffer()?).await?;
        let array_buffer = ArrayBuffer::from(js_value);
        let decoder = TextDecoder::new_with_label("utf-8")?;
        let content = decoder.decode_with_buffer_source(&array_buffer)?;
        let last = serde_json::from_str(&content)?;

        Ok(last)
    }

    pub async fn write(&self, key: String, val: Guess) -> Result<WriteResponse, PostError> {
        let mut headers = HashMap::new();
        headers.insert(
            "Authorization".to_string(),
            format!("Bearer {}", self.config.token),
        );
        headers.insert("Content-type".to_string(), "application/json".to_string());
        let url = format!(
            "https://api.cloudflare.com/client/v4/accounts/{}/storage/kv/namespaces/{}/values/{}",
            self.config.account_id, self.config.namespace_id, key
        );
        let req = Request {
            url: url,
            method: Method::PUT,
            headers: headers,
            body: val,
        };
        let js_resp = send(req).await?;

        // Convert this Promise into a rust Future.
        let json = JsFuture::from(js_resp.json()?).await?;

        let resp: WriteResponse = json.into_serde()?;
        Ok(resp)
    }
}

#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum WriteResponse {
    Ok(WriteSuccess),
    Err(WriteErr),
}

#[derive(Deserialize, Debug)]
pub struct WriteSuccess {
    pub success: bool,
    pub errors: Vec<String>,
    pub messages: Vec<String>,
}

#[derive(Deserialize, Debug)]
pub struct WriteErr {
    pub code: u16,
    pub error: String,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Guess {
    pub value: String,
    pub created_at: i64,
}

impl fmt::Display for Guess {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Create a NaiveDateTime from the timestamp
        let naive = NaiveDateTime::from_timestamp(self.created_at, 0);

        // Create a normal DateTime from the NaiveDateTime
        let readable_time: DateTime<Utc> = DateTime::from_utc(naive, Utc);
        write!(f, "{} submitted at {}", self.value, readable_time)
    }
}
