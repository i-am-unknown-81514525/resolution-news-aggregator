use chrono::{DateTime, FixedOffset};
use serde::{Serialize, Deserialize};
use common::unify::{SourceKind, UnifyOutput};

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct LocalUnify {
    pub idx: i64,
    pub id: String,
    pub organisation: String,
    pub title: String,
    pub description: String,
    #[serde(serialize_with = "serialize_dt", deserialize_with = "deserialize_dt")]
    pub time: DateTime<FixedOffset>,
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
            time: value.time,
            source: value.source,
            score: value.score,
            link: value.link
        }
    }
}

pub type Embedding = [f32; 384];