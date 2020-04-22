extern crate cfg_if;
#[macro_use]
extern crate serde_derive;
extern crate wasm_bindgen;
extern crate wasm_bindgen_futures;

mod post;
mod route;
mod slack;
#[macro_use]
mod utils;

use cfg_if::cfg_if;
use route::Route;
use slack::PostMessageResp;
use url::Url;
use uuid::Uuid;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsValue;
use wasm_bindgen_futures::JsFuture;
use web_sys::Request;

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
    #[serde(rename(deserialize = "type"))]
    ty: String,
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

#[derive(Deserialize, Debug)]
struct BotConfig {
    token: String,
    announcement_channel: String,
}

#[wasm_bindgen]
pub async fn interactive_bot(req: JsValue, bot_config: JsValue) -> Result<JsValue, JsValue> {
    let bot_config: BotConfig = bot_config.into_serde().map_err(|e| e.to_string())?;

    let req = Request::from(req);
    let url_str = req.url();
    let url = Url::parse(&url_str).map_err(|_| format!("{:?} is not a valid url", url_str))?;

    let path = url.path();
    match Route::from(path) {
        Route::CalendarStart => calendar_start(req, bot_config).await,
        Route::CalendarEnd => calendar_end(req).await,
        Route::Events => events(req).await,
        Route::Unhandled => Err(unhandled(path)),
    }?;
    Ok(JsValue::TRUE)
}

async fn calendar_start(req: Request, bot_config: BotConfig) -> Result<(), JsValue> {
    let body = JsFuture::from(req.json()?).await?;
    let resp = slack::post_message(
        "Emoji lottery begins".to_string(),
        bot_config.announcement_channel,
        bot_config.token,
    )
    .await?;
    match resp {
        PostMessageResp::Ok(_) => Ok(()),
        PostMessageResp::Err(e) => Err(JsValue::from_str(&e.error)),
    }
}

async fn calendar_end(req: Request) -> Result<(), JsValue> {
    Ok(())
}

async fn events(req: Request) -> Result<(), JsValue> {
    let body = JsFuture::from(req.json()?).await?;
    // Using into_serde API requires activating the serde-serialize feature of the wasm-bindgen crate.
    let event: MessageEvent = body.into_serde().map_err(|e| e.to_string())?;
    Ok(())
}
fn unhandled(path: &str) -> JsValue {
    JsValue::from_str(&format!("No handler defined for route {:?}", path))
}

// Expose a function to JS that generates v4 UUID
#[wasm_bindgen]
pub fn uuid() -> String {
    Uuid::new_v4().to_string()
}
