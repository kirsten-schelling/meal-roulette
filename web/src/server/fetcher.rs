use dioxus::prelude::*;
use serde::de::DeserializeOwned;
use std::error::Error;

pub fn get_url(route: &str) -> String {
    let api_url = std::env::var("API_URL").expect("API_URL must be set");
    format!("{}/{}", api_url, route)
}

pub async fn send_request<T>(
    method: impl FnOnce(&reqwest::Client) -> reqwest::RequestBuilder,
) -> Result<T, Box<dyn Error + Send + Sync>>
where
    T: DeserializeOwned,
{
    let client = reqwest::Client::new();
    let resp = method(&client).send().await?;

    // Read the body ONCE as text. `text()` consumes `resp`, so everything below
    // works off this `body` string — we never touch `resp` again.
    let status = resp.status();
    let body = resp.text().await?;

    // The backend answered with an error status: the body is an error message,
    // not a `T`. Surface it instead of failing to decode it as `T`.
    if !status.is_success() {
        eprintln!("[fetcher] backend returned {status}: {body}");
        return Err(format!("backend returned {status}: {body}").into());
    }

    serde_json::from_str::<T>(&body).map_err(|e| {
        eprintln!(
            "[fetcher] failed to decode {}: {e}\nbody was: {body}",
            std::any::type_name::<T>()
        );
        format!("decoding {}: {e}", std::any::type_name::<T>()).into()
    })
}
