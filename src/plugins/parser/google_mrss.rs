use serde::{Serialize, Deserialize, Deserializer};
use serde::de::{Visitor};
use crate::plugins::parser::common::DocumentID;
use crate::plugins::parser::utils::{string_as_rfc2822};

#[derive(Serialize, Deserialize, Debug)]
pub struct Origin {
    #[serde(rename = "@url")]
    pub url: String,
    #[serde(rename = "#text")]
    pub ublisher: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GoogleMrssItem {
    title: String,
    link: String,
    guid: DocumentID,
    #[serde(rename = "pubDate", deserialize_with = "string_as_rfc2822")]
    publish_date: chrono::DateTime<chrono::offset::FixedOffset>,
    source: Origin
}


#[derive(Serialize, Deserialize, Debug)]
pub struct GoogleMrssResult {
    generator: Option<String>,
    title: Option<String>,
    link: Option<String>,
    language: Option<String>,
    #[serde(rename = "lastBuildDate", deserialize_with = "string_as_rfc2822")]
    last_build_date: chrono::DateTime<chrono::offset::FixedOffset>,
    description: Option<String>,
    #[serde(rename = "item")]
    items: Vec<GoogleMrssItem>,
}
