use std::str::FromStr;
use crate::plugins::parser::utils::string_as_rfc2822;
use common::unify::{SourceKind, ToVecUnify, UnifyOutput};
use serde::Deserialize;
use url::Url;
use crate::plugins::parser::common::DocumentID;

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
pub struct Rss2RssItem {
    title: String,
    description: String,
    link: String,
    guid: DocumentID,
    #[serde(rename = "pubDate", deserialize_with = "string_as_rfc2822")]
    publish_date: chrono::DateTime<chrono::offset::FixedOffset>,
}

impl Rss2RssItem {
    pub fn get_unify(&self) -> UnifyOutput {
        UnifyOutput {
            idx: 0,
            id: self.guid.id.clone(),
            organisation: Url::from_str(&self.link).map(|v| v.clone().host_str().unwrap_or("Unknown").to_string()).unwrap_or("Unknown".to_string()),
            title: self.title.clone(),
            description: self.description.to_string(),
            time: self.publish_date,
            score: None,
            source: SourceKind::Origin,
            link: self.link.clone(),
            hash_key: vec![
                self.guid.id.clone(),
                format!("rss2:{}:{}:{}", self.title, self.description, self.link),
                format!("rss2:{}:{}:{}", self.title, self.description, self.publish_date.timestamp_micros()),
                format!("rss2:{}:{}", self.link, self.publish_date.timestamp_micros())
            ],
            embedding: None,
        }
    }
}

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
pub struct Rss2RssResult {
    title: Option<String>,
    link: Option<String>,
    description: Option<String>,
    #[serde(rename = "item", default)]
    items: Vec<Rss2RssItem>,
}

#[derive(Deserialize, Debug)]
pub struct Outer {
    pub channel: Rss2RssResult,
}

impl ToVecUnify for Rss2RssResult {
    fn to_vec_unify(&self) -> Vec<UnifyOutput> {
        self.items.iter().clone().map(|x| {
            let mut r = x.get_unify();
            if let Some(org_name) = self.description.clone() {
                r.organisation = org_name;
            };
            r
        }).collect()
    }
}

impl ToVecUnify for Outer {
    fn to_vec_unify(&self) -> Vec<UnifyOutput> {
        self.channel.to_vec_unify()
    }
}
