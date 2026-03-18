use once_cell::sync::Lazy;
use reqwest;
use serde::Deserialize;
use serde_xml_rs::from_str;
use thiserror::Error;
use tracing::info;

#[derive(Error, Debug)]
pub(crate) enum RssFetchError {
    #[error("RequestError")]
    RequestError(#[from] reqwest::Error),
    #[error("SerdeXmlParseError")]
    SerdeXmlParseError(#[from] serde_xml_rs::Error),
}

use std::sync::Arc;

use reqwest::ClientBuilder;
use reqwest_hickory_resolver::HickoryResolver;

fn init_with_hickory_resolver() -> reqwest::Result<reqwest::Client> {
    let mut builder = ClientBuilder::new();
    builder = builder.dns_resolver(Arc::new(HickoryResolver::default()));
    builder.build()
}

static CLIENT: Lazy<reqwest::Client> = Lazy::new(|| {
    init_with_hickory_resolver().unwrap() // reqwest::Client::new()
});

pub(crate) async fn get_raw(url: reqwest::Url) -> Result<String, RssFetchError> {
    info!(
        "Fetching url: {} {} (Host: {})",
        url,
        url.scheme(),
        match url.host() {
            Some(host) => host.to_string(),
            None => "N/A".to_string(),
        }
    );
    let req = CLIENT.get(url).send().await?;
    let resp = req.text().await.map_err(|e| RssFetchError::RequestError(e));
    resp
}

#[allow(dead_code)]
pub(crate) async fn fetch_rss<'a, T: Deserialize<'a>>(
    url: reqwest::Url,
) -> Result<T, RssFetchError> {
    let response = get_raw(url).await?;
    let parsed: T = from_str(&response).map_err(|e| RssFetchError::SerdeXmlParseError(e))?;
    Ok(parsed)
}
