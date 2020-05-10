use super::discord::new_webhook_client;
use super::emoji::LotteryConfig;
use super::kv::{KVClient, Response};
use super::slack::{new_slack_client, PostMessageResp};
use super::BotConfig;

use wasm_bindgen::JsValue;
use wasm_bindgen_futures::JsFuture;
use web_sys::Request;

#[derive(Debug, Deserialize)]
struct CalendarStartEvent {
    calendar_name: String,
}

pub enum NotifyTo {
    Discord,
    Slack,
}

pub async fn calendar_start(
    req: Request,
    bot_config: BotConfig,
    to: NotifyTo,
) -> Result<JsValue, JsValue> {
    let mention_maintainer = bot_config.mention_maintainer(&to);
    let body = JsFuture::from(req.json()?).await?;
    let event: CalendarStartEvent = body.into_serde().map_err(|e| {
        format!(
            "Failed to deserialize into CalendarStartEvent, err: {:?}",
            e
        )
    })?;
    let client = KVClient::new(bot_config.kv, bot_config.emoji.config_kv_namespace.clone());
    let key = bot_config.emoji.kv_key();
    let new_config = bot_config.emoji.commence();
    // Update emoji lottery config to start a new season
    let resp = client
        .write::<&LotteryConfig, ()>(&key, &new_config)
        .await
        .map_err(|e| format!("Failed to write new emoji lottery config, err: {:?}", e))?;

    let msg = match resp {
        Response::Ok(_) => format!(
            "{} season {} commence",
            event.calendar_name, new_config.season,
        ),
        Response::Err(e) => format!(
            "Failed to start new emoji lottery, err: {:?}\n {} please fix it.",
            e, mention_maintainer,
        ),
    };

    match &to {
        NotifyTo::Slack => {
            let slack_client = new_slack_client(bot_config.slack);
            let resp = slack_client.post_message(msg).await?;
            match resp {
                PostMessageResp::Ok(_) => Ok(JsValue::TRUE),
                PostMessageResp::Err(e) => Err(JsValue::from_str(&e.error)),
            }
        }
        NotifyTo::Discord => {
            let webhook_client = new_webhook_client(bot_config.discord.webhook_url);
            webhook_client.execute(msg).await?;
            Ok(JsValue::TRUE)
        }
    }
}

pub async fn calendar_end(req: Request) -> Result<JsValue, JsValue> {
    Ok(JsValue::TRUE)
}
