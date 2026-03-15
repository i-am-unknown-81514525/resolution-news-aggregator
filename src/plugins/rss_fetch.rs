use reqwest;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use serde_xml_rs::{from_str};

#[derive(Error, Debug)]
pub(crate) enum RssFetchError {
    RequestError(#[from] reqwest::Error),
    SerdeXmlParseError(serde_xml_rs::Error)
}

static CLIENT: Lazy<reqwest::Client> = Lazy::new(|| {
    reqwest::Client::new()
});

pub(crate) async fn get_raw(url: reqwest::Url) -> Result<String, RssFetchError> {
    CLIENT.get(url).send().await?.text().await.map_err(|e| RssFetchError::RequestError(e))
}

pub(crate) async fn fetch_rss<T: Deserialize>(url: reqwest::Url) -> Result<T, RssFetchError> {
    let response = get_raw(url).await?;
    let parsed: T = from_str(&response).map_err(|e| RssFetchError::SerdeXmlParseError(e));
    Ok(parsed)
}