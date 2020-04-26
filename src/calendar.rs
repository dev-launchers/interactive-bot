use super::discord::new_webhook_client;
use super::slack::{new_slack_client, PostMessageResp};
use super::BotConfig;

use wasm_bindgen::JsValue;
use wasm_bindgen_futures::JsFuture;
use web_sys::Request;

#[derive(Debug, Deserialize)]
struct CalendarStartEvent {
    calendar_name: String,
}

pub enum notifyTo {
    Discord,
    Slack,
}

pub async fn calendar_start(
    req: Request,
    bot_config: BotConfig,
    to: notifyTo,
) -> Result<(), JsValue> {
    let body = JsFuture::from(req.json()?).await?;
    let event: CalendarStartEvent = body.into_serde().map_err(|e| {
        format!(
            "Failed to deserialize into CalendarStartEvent, err: {:?}",
            e
        )
    })?;

    let msg = format!("{} commence", event.calendar_name);
    match to {
        notifyTo::Slack => {
            let slack_client = new_slack_client(bot_config.slack);
            let resp = slack_client.post_message(msg).await?;
            match resp {
                PostMessageResp::Ok(_) => Ok(()),
                PostMessageResp::Err(e) => Err(JsValue::from_str(&e.error)),
            }
        }
        notifyTo::Discord => {
            let webhook_client = new_webhook_client(bot_config.discord.webhook_url);
            webhook_client.execute(msg).await?;
            Ok(())
        }
    }
}

pub async fn calendar_end(req: Request) -> Result<(), JsValue> {
    Ok(())
}
