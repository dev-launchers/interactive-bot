extern crate cfg_if;
extern crate reqwest;
#[macro_use]
extern crate serde_derive;
extern crate wasm_bindgen;
extern crate wasm_bindgen_futures;
mod utils;

use cfg_if::cfg_if;
use std::collections::HashMap;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::{future_to_promise, spawn_local};

// similar to the if/elif C preprocessor macro
cfg_if! {
    // When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
    // allocator.
    if #[cfg(feature = "wee_alloc")] {
        extern crate wee_alloc;
        #[global_allocator]
        static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;
    }
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

macro_rules! console_log {
    // Note that this is using the `log` function imported above during
    // `bare_bones`
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
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

#[wasm_bindgen]
pub async fn lottery(event: JsValue, token: JsValue) -> Result<JsValue, JsValue> {
    console_log!("event {:?}", event);
    // JsValue.into_serde invokes JSON.stringify on this value and then parses the
    // resulting JSON into an arbitrary Rust value.
    // Using this API requires activating the serde-serialize feature of the wasm-bindgen crate.
    let event = event.into_serde().map_err(|e| e.to_string())?;
    console_log!("event is {:?}", event);
    Ok(JsValue::TRUE)
    // https://github.com/rustwasm/wasm-bindgen/issues/1126
    //post_message_async(token);
    //post_message_future(token).await
    /*spawn_local(async {
        console_log!("start spawn_local");
        match post_message(token).await {
            Ok(_) => console_log!("post_message ok"),
            Err(e) => console_log!("post_message err {}", e),
        };
        console_log!("end spawn_local");
    })*/
    /*async {
        console_log!("start spawn_local");
        match post_message(token).await {
            Ok(_) => console_log!("post_message ok"),
            Err(e) => console_log!("post_message err {}", e),
        };
        console_log!("end spawn_local");
    })*/
}

/*
fn post_message(token: String) {
    // reqwest::blocking not found when compiling with cargo build --lib --release --target wasm32-unknown-unknown
    // works fine when compiling without specifying target
    let client = reqwest::blocking::Client::new();
    let post_api = "https://slack.com/api/chat.postMessage".to_string();
    let mut req_param = HashMap::new();
    req_param.insert("channel", "CNHFQSXA7".to_string());
    req_param.insert("text", "Hello, I'm lottery bot".to_string());
    console_log!("start spawn_local");
    match client
        .post(&post_api)
        .header("Authorization", format!("Bearer {}", token))
        .json(&req_param)
        .send()
    {
        Ok(_) => console_log!("post_message ok"),
        Err(e) => console_log!("post_message err {}", e),
    };
}
*/
//token: &'static str
fn post_message_async(token: String) {
    spawn_local(async {
        let client = reqwest::Client::new();
        let post_api = "https://slack.com/api/chat.postMessage".to_string();
        let mut req_param = HashMap::new();
        req_param.insert("channel", "CNHFQSXA7".to_string());
        req_param.insert("text", "Hello, I'm lottery bot".to_string());
        console_log!("start spawn_local");
        match client
            .post(&post_api)
            .header(
                "Authorization",
                format!(
                    "Bearer {}",
                    "xoxb-732916575875-976244796035-JvvdbaysxdGnSGzAQ4cOyAne"
                ),
            )
            .json(&req_param)
            .send()
            .await
        {
            Ok(_) => console_log!("post_message_async ok"),
            Err(e) => console_log!("post_message_async err {}", e),
        };
        console_log!("end spawn_local");
    })
}

#[wasm_bindgen]
pub async fn post_message_future(token: JsValue) -> Result<JsValue, JsValue> {
    let client = reqwest::Client::new();
    let post_api = "https://slack.com/api/chat.postMessage".to_string();
    let mut req_param = HashMap::new();
    req_param.insert("channel", "CNHFQSXA7".to_string());
    req_param.insert("text", "Hello, I'm lottery bot".to_string());
    let res = match client
        .post(&post_api)
        .header(
            "Authorization",
            format!(
                "Bearer {}",
                "xoxb-732916575875-976244796035-JvvdbaysxdGnSGzAQ4cOyAne"
            ),
        )
        .json(&req_param)
        .send()
        .await
    {
        Ok(body) => body,
        Err(e) => {
            console_log!("post_message_future send error {}", e);
            return Err(JsValue::from_str("send error"));
        }
    };
    Ok(JsValue::NULL)
    /*match res.json().await {
        Ok(_) => Ok(()),
        Err(e) => {
            console_log!("post_message_future read body error {}", e);
            return Err(JsValue::from_str("read body error"));
        }
    }*/
}
