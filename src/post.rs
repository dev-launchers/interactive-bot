use super::slack::PostMessageResp;
use serde::Serialize;
use std::collections::HashMap;
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use web_sys;

pub struct Request<T: ?Sized>
where
    T: Serialize,
{
    pub url: String,
    pub headers: HashMap<String, String>,
    pub body: T,
}

pub async fn post<T>(req: Request<T>) -> Result<PostMessageResp, PostError>
where
    T: Serialize,
{
    let mut opts = web_sys::RequestInit::new();
    opts.method("POST");
    let body = JsValue::from_serde(&req.body)?;
    opts.body(Some(&body));

    let request = web_sys::Request::new_with_str_and_init(&req.url, &opts)?;
    /*
          warning: unused `std::iter::Map` that must be used
      --> src/post.rs:26:5
       |
    26 | /     req.headers
    27 | |         .iter()
    28 | |         .map(|(k, v)| request.headers().set(k, v).unwrap());
       | |____________________________________________________________^
       |
       = note: `#[warn(unused_must_use)]` on by default
       = note: iterators are lazy and do nothing unless consumed

    */
    for (k, v) in req.headers.iter() {
        request.headers().set(k, v)?;
    }

    let window = web_sys::window().ok_or(PostError::NoWindow)?;

    // `resp_value` is a JS `Response` object.
    let resp_value = JsFuture::from(window.fetch_with_request(&request)).await?;

    let resp: web_sys::Response = resp_value.dyn_into()?;

    // Convert this other Promise into a rust Future.
    let json = JsFuture::from(resp.json()?).await?;

    let response: PostMessageResp = json.into_serde()?;

    Ok(response)
}

#[derive(Debug)]
pub enum PostError {
    Jv(JsValue),
    SerdeJson(serde_json::error::Error),
    NoWindow,
}

impl From<PostError> for JsValue {
    fn from(pe: PostError) -> Self {
        match pe {
            PostError::Jv(jv) => jv,
            PostError::SerdeJson(e) => JsValue::from_str(&e.to_string()),
            PostError::NoWindow => {
                JsValue::from_str("The runtime doesn't expose a Window interface")
            }
        }
    }
}

impl From<serde_json::error::Error> for PostError {
    fn from(e: serde_json::error::Error) -> Self {
        PostError::SerdeJson(e)
    }
}

impl From<JsValue> for PostError {
    fn from(e: JsValue) -> Self {
        PostError::Jv(e)
    }
}
