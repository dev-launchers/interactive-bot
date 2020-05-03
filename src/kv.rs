use super::http::{send, Method, PostError, Request};
use chrono::prelude::*;
use js_sys::ArrayBuffer;
use serde::Serialize;
use serde_json;
use std::collections::HashMap;
use std::fmt;
use wasm_bindgen::JsValue;
use wasm_bindgen_futures::JsFuture;
use web_sys::TextDecoder;

#[derive(Deserialize)]
pub struct KVConfig {
    token: String,
    account_id: String,
}

pub struct KVClient {
    token: String,
    account_id: String,
    namespace_id: String,
}

impl KVClient {
    pub fn new(config: KVConfig, namespace_id: String) -> KVClient {
        KVClient {
            token: config.token,
            account_id: config.account_id,
            namespace_id,
        }
    }

    pub async fn read(&self, key: &str) -> Result<Option<Guess>, PostError> {
        let mut headers = HashMap::new();
        headers.insert(
            "Authorization".to_string(),
            format!("Bearer {}", self.token),
        );
        headers.insert("Content-type".to_string(), "application/json".to_string());
        let url = format!(
            "https://api.cloudflare.com/client/v4/accounts/{}/storage/kv/namespaces/{}/values/{}",
            self.account_id, self.namespace_id, key
        );
        let req = Request {
            url: url,
            method: Method::GET,
            headers: headers,
            body: (),
        };
        let js_resp = send(req).await?;

        if js_resp.ok() {
            // Convert this Promise into a rust Future.
            let js_value = JsFuture::from(js_resp.array_buffer()?).await?;
            let array_buffer = ArrayBuffer::from(js_value);
            let decoder = TextDecoder::new_with_label("utf-8")?;
            let content = decoder.decode_with_buffer_source(&array_buffer)?;
            let last: Guess = serde_json::from_str(&content)?;
            Ok(Some(last))
        } else {
            // Convert this Promise into a rust Future.
            let json = JsFuture::from(js_resp.json()?).await?;
            let resp: ReadErr = json.into_serde()?;
            let errCode = resp.errors[0].code;
            // 10009 is key not found
            if errCode == 10009 {
                return Ok(None);
            }
            return Err(PostError::Jv(JsValue::from_str(&format!(
                "Failed to read key, err code {}, message {}",
                errCode, resp.errors[0].message
            ))));
        }
    }

    pub async fn write<T>(&self, key: &str, val: T) -> Result<WriteResponse, PostError>
    where
        T: Serialize,
    {
        let mut headers = HashMap::new();
        headers.insert(
            "Authorization".to_string(),
            format!("Bearer {}", self.token),
        );
        headers.insert("Content-type".to_string(), "application/json".to_string());
        let url = format!(
            "https://api.cloudflare.com/client/v4/accounts/{}/storage/kv/namespaces/{}/values/{}",
            self.account_id, self.namespace_id, key
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

#[derive(Deserialize, Debug)]
pub struct ReadErr {
    errors: Vec<ErrCode>,
}

#[derive(Deserialize, Debug)]
pub struct ErrCode {
    code: u64,
    message: String,
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
