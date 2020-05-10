use super::error;
use super::http::{send, Method, Request};

use chrono::prelude::*;
use js_sys::ArrayBuffer;
use serde::{de::DeserializeOwned, Serialize};
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

    pub async fn read(&self, key: &str) -> Result<Option<Guess>, error::Error> {
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
            let resp = json.into_serde::<NoResultResponse>()?;
            let err_code = resp.errors[0].code;
            // 10009 is key not found
            if err_code == 10009 {
                return Ok(None);
            }
            return Err(error::Error::Jv(JsValue::from_str(&format!(
                "Failed to read key, err code {}, message {}",
                err_code, resp.errors[0].message
            ))));
        }
    }

    pub async fn write<T>(&self, key: &str, val: T) -> Result<(), error::Error>
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

        let resp = json.into_serde::<NoResultResponse>()?;
        resp.result()
    }

    pub async fn create_namespace(
        &self,
        title: String,
    ) -> Result<CreateNamespaceResult, error::Error> {
        let mut headers = HashMap::new();
        headers.insert(
            "Authorization".to_string(),
            format!("Bearer {}", self.token),
        );
        headers.insert("Content-type".to_string(), "application/json".to_string());
        let url = format!(
            "https://api.cloudflare.com/client/v4/accounts/{}/storage/kv/namespaces",
            self.account_id
        );
        let req = Request {
            url: url,
            method: Method::POST,
            headers: headers,
            body: CreateNamespaceBody { title: title },
        };
        let js_resp = send(req).await?;

        // Convert this Promise into a rust Future.
        let json = JsFuture::from(js_resp.json()?).await?;

        let resp = json.into_serde::<ExpectResultResponse<CreateNamespaceResult>>()?;

        resp.result()
    }

    pub async fn delete_namespace(&self, namespace_id: String) -> Result<(), error::Error> {
        let mut headers = HashMap::new();
        headers.insert(
            "Authorization".to_string(),
            format!("Bearer {}", self.token),
        );
        headers.insert("Content-type".to_string(), "application/json".to_string());
        let url = format!(
            "https://api.cloudflare.com/client/v4/accounts/{}/storage/kv/namespaces/{}",
            self.account_id, namespace_id
        );
        let req = Request {
            url: url,
            method: Method::DELETE,
            headers: headers,
            body: (),
        };
        let js_resp = send(req).await?;

        // Convert this Promise into a rust Future.
        let json = JsFuture::from(js_resp.json()?).await?;

        let resp = json.into_serde::<NoResultResponse>()?;

        resp.result()
    }
}

#[derive(Deserialize, Debug)]
pub struct ExpectResultResponse<T>
where
    T: Clone,
{
    pub result: Option<T>,
    pub success: bool,
    pub errors: Vec<Error>,
    pub messages: Vec<String>,
}

impl<T> ExpectResultResponse<T>
where
    T: Clone,
{
    pub fn result(&self) -> Result<T, error::Error> {
        if self.errors.len() > 0 {
            Err(error::Error::from(self.errors[0].clone()))
        } else {
            match &self.result {
                Some(r) => Ok(r.clone()),
                None => Err(error::Error::KvNoResult),
            }
        }
    }
}

#[derive(Deserialize, Debug)]
pub struct NoResultResponse {
    pub success: bool,
    pub errors: Vec<Error>,
    pub messages: Vec<String>,
}

impl NoResultResponse {
    pub fn result(&self) -> Result<(), error::Error> {
        if self.errors.len() > 0 {
            Err(error::Error::from(self.errors[0].clone()))
        } else {
            Ok(())
        }
    }
}

#[derive(Deserialize, Clone, Debug)]
pub struct Error {
    pub code: u16,
    pub message: String,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "kv error, code: {}, message: {}",
            self.code, self.message
        )
    }
}

#[derive(Serialize, Debug)]
struct CreateNamespaceBody {
    title: String,
}

#[derive(Deserialize, Clone, Debug)]
pub struct CreateNamespaceResult {
    pub id: String,
    title: String,
    supports_url_encoding: bool,
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
