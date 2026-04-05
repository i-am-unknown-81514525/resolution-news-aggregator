use core::error;
use chrono::{DateTime, FixedOffset};
use rkyv::{Serialize, Deserialize, Archive};
use common::unify::{UnifyOutput};

#[derive(Clone, Serialize, Deserialize, Debug, Archive)]
pub enum SourceKind {
    LinkedSource(String, String),
    Source(String),
    Origin,
    Unknown
}

#[derive(Clone, Serialize, Deserialize, Debug, Archive)]
pub struct LocalUnify {
    pub idx: i64,
    pub id: String,
    pub organisation: String,
    pub title: String,
    pub description: String,
    pub time: i64,
    pub source: SourceKind, // This describes where the content was received from
    pub score: Option<f32>, // Score for importance of the news
    pub link: String,
}

impl From<UnifyOutput> for LocalUnify {
    fn from(value: UnifyOutput) -> Self {
        Self {
            idx: value.idx,
            id: value.id,
            organisation: value.organisation,
            title: value.title,
            description: value.description,
            time: value.time.timestamp(),
            source: match value.source {
                common::unify::SourceKind::LinkedSource(s, l) => SourceKind::LinkedSource(s, l),
                common::unify::SourceKind::Source(s) => SourceKind::Source(s),
                common::unify::SourceKind::Origin => SourceKind::Origin,
                common::unify::SourceKind::Unknown => SourceKind::Unknown
            },
            score: value.score,
            link: value.link
        }
    }
}

#[derive(Clone, Serialize, Deserialize, Debug, Archive, Copy)]
pub struct Embedding([f32; 384]);