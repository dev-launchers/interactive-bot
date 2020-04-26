use super::slack;
use super::BotConfig;

use wasm_bindgen::JsValue;
use wasm_bindgen_futures::JsFuture;
use web_sys::Request;

#[derive(Debug, Deserialize)]
struct CalendarStartEvent {
    calendar_name: String,
}

pub async fn calendar_start(req: Request, bot_config: BotConfig) -> Result<(), JsValue> {
    let body = JsFuture::from(req.json()?).await?;
    let event: CalendarStartEvent = body.into_serde().map_err(|e| {
        format!(
            "Failed to deserialize into CalendarStartEvent, err: {:?}",
            e
        )
    })?;
    let resp = slack::post_message(
        format!("{} commence", event.calendar_name),
        bot_config.slack.announcement_channel,
        bot_config.slack.token,
    )
    .await?;
    match resp {
        slack::PostMessageResp::Ok(_) => Ok(()),
        slack::PostMessageResp::Err(e) => Err(JsValue::from_str(&e.error)),
    }
}

pub async fn calendar_end(req: Request) -> Result<(), JsValue> {
    Ok(())
}
