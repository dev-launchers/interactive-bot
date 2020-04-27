use super::post::{post, PostError, Request};
use std::collections::HashMap;
use wasm_bindgen_futures::JsFuture;

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

    pub async fn read(&self, key: String) -> Result<Guess, PostError> {
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
            headers: headers,
            body: (),
        };
        let js_resp = post(req).await?;

        // Convert this Promise into a rust Future.
        let json = JsFuture::from(js_resp.json()?).await?;

        let resp: Guess = json.into_serde()?;
        Ok(resp)
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
            headers: headers,
            body: val,
        };
        let js_resp = post(req).await?;

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
    pub created_at: u64,
}
