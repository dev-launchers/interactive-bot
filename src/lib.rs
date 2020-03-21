extern crate cfg_if;
#[macro_use]
extern crate serde_derive;
extern crate wasm_bindgen;
extern crate wasm_bindgen_futures;

mod post;
mod slack;
mod utils;

use cfg_if::cfg_if;
use wasm_bindgen::prelude::*;

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

// https://api.slack.com/methods/chat.postMessage
#[derive(Serialize, Debug)]
struct PostMessageBody {
    channel: String,
    text: String,
    as_user: Option<bool>,
}

#[wasm_bindgen]
pub async fn lottery(body: JsValue, token: JsValue) -> Result<JsValue, JsValue> {
    // JsValue.into_serde invokes JSON.stringify on this value and then parses the
    // resulting JSON into an arbitrary Rust value.
    // Using this API requires activating the serde-serialize feature of the wasm-bindgen crate.
    let event: MessageEvent = body.into_serde().map_err(|e| e.to_string())?;
    // Issue: If I don't annotate the type of event, compiler returns, I got this run time error
    // invalid type: map, expected unit at line 1 column 0
    console_log!("event is {:?}", event);
    let resp = slack::post_message(token.as_string().unwrap()).await?;
    if resp.ok {
        return Ok(JsValue::TRUE);
    }
    match resp.error {
        Some(e) => Err(JsValue::from_str(&e.to_string())),
        None => Err(JsValue::from_str("Slack didn't return failure reason")),
    }
}
