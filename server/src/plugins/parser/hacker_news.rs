use std::str::FromStr;
use crate::plugins::parser::common::DocumentID;
use crate::plugins::parser::utils::string_as_rfc2822;
use common::unify::{SourceKind, ToVecUnify, UnifyOutput};
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
pub struct HackerNewsRssItem {
    title: String,
    link: String,
    #[serde(rename = "pubDate", deserialize_with = "string_as_rfc2822")]
    publish_date: chrono::DateTime<chrono::offset::FixedOffset>,
    comments: String,
}

impl HackerNewsRssItem {
    pub fn get_unify(&self) -> UnifyOutput {
        UnifyOutput {
            id: format!("hacker_news:{}", self.comments.clone().replace("https://news.ycombinator.com/item?id=", "")),
            organisation: Url::from_str(&self.link).map(|v| v.clone().host_str().unwrap_or("Unknown").to_string()).unwrap_or("Unknown".to_string()),
            title: self.title.clone(),
            description: "".to_string(),
            time: self.publish_date,
            score: None,
            source: SourceKind::LinkedSource("Hacker News".to_string(), self.link.clone()),
            link: self.link.clone()
        }
    }
}

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
pub struct HackerNewsRssResult {
    title: Option<String>,
    link: Option<String>,
    description: Option<String>,
    #[serde(rename = "item", default)]
    items: Vec<HackerNewsRssItem>,
}

#[derive(Deserialize, Debug)]
pub struct Outer {
    pub channel: HackerNewsRssResult,
}

impl ToVecUnify for HackerNewsRssResult {
    fn to_vec_unify(&self) -> Vec<UnifyOutput> {
        self.items.iter().clone().map(|x| x.get_unify()).collect()
    }
}

impl ToVecUnify for Outer {
    fn to_vec_unify(&self) -> Vec<UnifyOutput> {
        self.channel.to_vec_unify()
    }
}
