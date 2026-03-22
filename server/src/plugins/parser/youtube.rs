use common::unify::{SourceKind, ToVecUnify, UnifyOutput};
use crate::plugins::parser::utils::string_as_rfc3339;
use serde::Deserialize;

#[derive(Deserialize, Debug, Default)]
pub struct Author {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub uri: String,
}

#[derive(Deserialize, Debug, Default)]
pub struct Link {
    #[serde(rename = "@rel", default)]
    pub rel: String,
    #[serde(rename = "@href", default)]
    pub href: String,
}

#[derive(Deserialize, Debug, Default)]
pub struct MediaGroup {
    #[serde(rename = "media:title", default)]
    pub title: String,
    #[serde(rename = "media:description", default)]
    pub description: String,
}

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
pub struct YoutubeFeedEntry {
    pub id: String,
    #[serde(rename = "yt:videoId")]
    pub video_id: String,
    #[serde(rename = "yt:channelId")]
    pub channel_id: String,
    pub title: String,
    pub link: Link,
    #[serde(default)]
    pub author: Author,
    #[serde(deserialize_with = "string_as_rfc3339")]
    pub published: chrono::DateTime<chrono::offset::FixedOffset>,
    #[serde(deserialize_with = "string_as_rfc3339")]
    pub updated: chrono::DateTime<chrono::offset::FixedOffset>,
    #[serde(rename = "media:group", default)]
    pub media_group: MediaGroup,
}

impl YoutubeFeedEntry {
    pub fn get_unify(&self) -> UnifyOutput {
        let id = format!("youtube:{}", self.video_id);
        UnifyOutput {
            id: id.clone(),
            organisation: self.author.name.clone(),
            title: self.title.clone(),
            description: self.media_group.description.clone(),
            time: self.published,
            score: None,
            source: SourceKind::LinkedSource("YouTube".to_string(), self.author.uri.clone()),
            link: self.link.href.clone(),
            hash_key: vec![id]
        }
    }
}

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
pub struct Outer {
    #[serde(rename = "entry", default)]
    pub entries: Vec<YoutubeFeedEntry>,
}

impl ToVecUnify for Outer {
    fn to_vec_unify(&self) -> Vec<UnifyOutput> {
        self.entries.iter().map(|x| x.get_unify()).collect()
    }
}
