use crate::plugins::parser::common::DocumentID;
use crate::plugins::parser::utils::string_as_rfc2822;
use common::unify::{SourceKind, ToVecUnify, UnifyOutput};
use serde::{Deserialize, Serialize};
use tower_http::CompressionLevel::Default;

#[derive(Serialize, Deserialize, Debug)]
pub struct Origin {
    #[serde(rename = "@url")]
    pub url: String,
    #[serde(rename = "#text")]
    pub publisher: String,
}

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
pub struct GoogleMrssItem {
    title: String,
    link: String,
    guid: DocumentID,
    #[serde(rename = "pubDate", deserialize_with = "string_as_rfc2822")]
    publish_date: chrono::DateTime<chrono::offset::FixedOffset>,
    source: Origin,
}

impl GoogleMrssItem {
    pub fn get_unify(&self) -> UnifyOutput {
        UnifyOutput {
            id: self.guid.id.clone(),
            organisation: self.source.publisher.clone(),
            title: self.title.clone(),
            description: "".to_string(),
            time: self.publish_date,
            score: None,
            source: SourceKind::Source("Google News".to_string()),
            link: self.link.clone(),
            hash_key: vec![self.guid.id.clone()]
        }
    }
}

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
pub struct GoogleMrssResult {
    generator: Option<String>,
    title: Option<String>,
    link: Option<String>,
    language: Option<String>,
    #[serde(rename = "lastBuildDate", deserialize_with = "string_as_rfc2822")]
    last_build_date: chrono::DateTime<chrono::offset::FixedOffset>,
    description: Option<String>,
    #[serde(rename = "item", default)]
    items: Vec<GoogleMrssItem>,
}

#[derive(Deserialize, Debug)]
pub struct Outer {
    pub channel: GoogleMrssResult,
}

impl ToVecUnify for GoogleMrssResult {
    fn to_vec_unify(&self) -> Vec<UnifyOutput> {
        self.items.iter().clone().map(|x| x.get_unify()).collect()
    }
}

impl ToVecUnify for Outer {
    fn to_vec_unify(&self) -> Vec<UnifyOutput> {
        self.channel.to_vec_unify()
    }
}
