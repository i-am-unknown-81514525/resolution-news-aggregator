

use common::unify::{SourceKind, ToVecUnify, UnifyOutput};
use crate::plugins::parser::utils::{string_as_rfc3339, html_to_text};
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
    #[serde(rename = "@href", default)]
    pub href: String,
}

#[derive(Deserialize, Debug, Default)]
pub struct Category {
    #[serde(rename = "@term", default)]
    pub term: String,
    #[serde(rename = "@label", default)]
    pub label: String,
}

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
pub struct RedditFeedEntry {
    pub id: String, //
    pub title: String, //
    pub link: Link, //
    #[serde(default)]
    pub author: Author, //
    #[serde(deserialize_with = "string_as_rfc3339")]
    pub published: chrono::DateTime<chrono::offset::FixedOffset>,
    #[serde(deserialize_with = "string_as_rfc3339")]
    pub updated: chrono::DateTime<chrono::offset::FixedOffset>,
    #[serde(deserialize_with = "html_to_text")]
    pub content: Option<String>,
    pub category: Category
}

impl RedditFeedEntry {
    pub fn get_unify(&self) -> UnifyOutput {
        let id = format!("reddit:{}", self.id);
        UnifyOutput {
            id: id.clone(),
            organisation: format!("{} in {}", self.author.name.clone(), self.category.label),
            title: self.title.clone(),
            description: self.content.clone().unwrap_or(String::from("")),
            time: self.published,
            score: None,
            source: SourceKind::LinkedSource("Reddit".to_string(), self.author.uri.clone()),
            link: self.link.href.clone(),
            hash_key: vec![id]
        }
    }
}

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
pub struct Outer {
    #[serde(rename = "entry", default)]
    pub entries: Vec<RedditFeedEntry>,
}

impl ToVecUnify for Outer {
    fn to_vec_unify(&self) -> Vec<UnifyOutput> {
        self.entries.iter().map(|x| x.get_unify()).collect()
    }
}
