use chrono::{DateTime,offset::Utc};
use async_trait::async_trait;
use serde::{Serialize, Serializer};

fn seralize_dt<S>(x: &DateTime<chrono::offset::FixedOffset>, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    s.serialize_f64(x.naive_utc().and_utc().timestamp_micros() as f64 / (1_000_000.0f64))
}

/// A unified output format to be displayed on the websocket
#[derive(Serialize, Debug)]
pub struct UnifyOutput {
    pub(crate) organisation: String,
    pub(crate) title: String,
    pub(crate) description: String,
    #[serde(serialize_with = "seralize_dt")]
    pub(crate) time: DateTime<chrono::offset::FixedOffset>,
    pub(crate) score: Option<f32> // Score for importance of the news
}


// pub trait Config {}

//
// #[async_trait]
// pub trait NewsScraper<T: Config> {
//     fn from(config: T) -> Box<Self>;
//
//     async fn retrieve(self) -> Vec<UnifyOutput>;
// }

pub trait ToVecUnify {
    fn to_vec_unify(&self) -> Vec<UnifyOutput>;
}