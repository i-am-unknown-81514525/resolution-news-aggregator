use reqwest;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use serde_xml_rs::{from_str};
use tracing::info;

#[derive(Error, Debug)]
pub(crate) enum RssFetchError {
    #[error("RequestError")]
    RequestError(#[from] reqwest::Error),
    #[error("SerdeXmlParseError")]
    SerdeXmlParseError(#[from] serde_xml_rs::Error)
}

static CLIENT: Lazy<reqwest::Client> = Lazy::new(|| {
    reqwest::Client::new()
});

pub(crate) async fn get_raw(url: reqwest::Url) -> Result<String, RssFetchError> {
    info!("Fetching url: {} {} (Host: {})", url, url.scheme(), match url.host() {
        Some(host) => host.to_string(),
        None => "N/A".to_string(),
    });
    let req = CLIENT.get(url).send().await?;
    let resp = req.text().await.map_err(|e| RssFetchError::RequestError(e));
    resp
}

pub(crate) async fn fetch_rss<'a, T: Deserialize<'a>>(url: reqwest::Url) -> Result<T, RssFetchError> {
    let response = get_raw(url).await?;
    let parsed: T = from_str(&response).map_err(|e| RssFetchError::SerdeXmlParseError(e))?;
    Ok(parsed)
}