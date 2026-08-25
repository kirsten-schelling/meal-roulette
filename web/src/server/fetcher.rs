use dioxus::prelude::*;
use serde::de::DeserializeOwned;

pub fn get_url(route: &str) -> String {
    let api_url = std::env::var("API_URL").expect("API_URL must be set");
    format!("{}/{}", api_url, route)
}

pub async fn send_request<T>(
    method: impl FnOnce(&reqwest::Client) -> reqwest::RequestBuilder,
) -> Result<T, reqwest::Error>
where
    T: DeserializeOwned,
{
    let client = reqwest::Client::new();
    method(&client).send().await?.json::<T>().await
}
