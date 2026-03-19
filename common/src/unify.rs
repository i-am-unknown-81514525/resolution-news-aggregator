use bytes::Bytes;
use chrono::DateTime;
use serde::{Serialize, Serializer};

fn seralize_dt<S>(x: &DateTime<chrono::offset::FixedOffset>, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    s.serialize_f64(x.naive_utc().and_utc().timestamp_micros() as f64 / (1_000_000.0f64))
}

/// A unified output format to be displayed on the websocket
#[derive(Serialize, Debug, Clone)]
pub struct UnifyOutput {
    pub organisation: String,
    pub title: String,
    pub description: String,
    #[serde(serialize_with = "seralize_dt")]
    pub time: DateTime<chrono::offset::FixedOffset>,
    pub score: Option<f32>, // Score for importance of the news
}

impl UnifyOutput {
    pub fn to_raw(&self) -> UnifyOutputRaw {
        UnifyOutputRaw {
            data: Bytes::from(serde_json::to_string(self).unwrap()),
        }
    }
}

#[derive(Clone, Debug)]
pub struct UnifyOutputRaw {
    pub data: Bytes,
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
