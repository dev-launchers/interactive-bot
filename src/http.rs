use super::error::Error;

use serde::Serialize;
use std::collections::HashMap;
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use web_sys::{RequestInit, Response};

pub struct Request<T: ?Sized>
where
    T: Serialize,
{
    pub url: String,
    pub method: Method,
    pub headers: HashMap<String, String>,
    pub body: T,
}

pub enum Method {
    GET,
    POST,
    PUT,
    DELETE,
}

impl Method {
    pub fn as_str(&self) -> &str {
        match self {
            Method::GET => "GET",
            Method::POST => "POST",
            Method::PUT => "PUT",
            Method::DELETE => "DELETE",
        }
    }
}

pub async fn send<T>(req: Request<T>) -> Result<Response, Error>
where
    T: Serialize,
{
    let mut opts = RequestInit::new();

    let method = req.method;
    opts.method(method.as_str());
    match method {
        Method::GET | Method::DELETE => {}
        Method::POST | Method::PUT => {
            // Equivalent to JSON.stringify in JS
            let body = JsValue::from_str(&serde_json::to_string(&req.body)?);
            opts.body(Some(&body));
        }
    };

    let request = web_sys::Request::new_with_str_and_init(&req.url, &opts)?;
    for (k, v) in req.headers.iter() {
        request.headers().set(k, v)?;
    }

    let window = worker_global_scope().ok_or(Error::NoWindow)?;

    // `resp_value` is a JS `Response` object.
    let resp_value = JsFuture::from(window.fetch_with_request(&request)).await?;

    let resp: Response = resp_value.dyn_into()?;
    Ok(resp)
}

// Returns global execution context of a service worker
fn worker_global_scope() -> Option<web_sys::ServiceWorkerGlobalScope> {
    js_sys::global()
        .dyn_into::<web_sys::ServiceWorkerGlobalScope>()
        .ok()
}
