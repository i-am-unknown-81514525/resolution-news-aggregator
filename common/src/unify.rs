use std::fmt;
use chrono::{DateTime, FixedOffset, TimeZone, Utc};
use serde::{de, Deserialize, Deserializer, Serialize, Serializer};

fn serialize_dt<S>(x: &DateTime<FixedOffset>, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    s.serialize_f64(x.naive_utc().and_utc().timestamp_micros() as f64 / (1_000_000.0f64))
}

pub fn deserialize_dt<'de, D>(deserializer: D) -> Result<DateTime<FixedOffset>, D::Error>
where
    D: Deserializer<'de>,
{
    let epoch = f64::deserialize(deserializer)?;

    let seconds = epoch.trunc() as i64;
    let nanoseconds = (epoch.fract() * 1_000_000_000.0).round() as u32;

    // Create a Utc DateTime first, then convert to FixedOffset (defaults to +00:00)
    let datetime = Utc
        .timestamp_opt(seconds, nanoseconds)
        .single()
        .ok_or_else(|| serde::de::Error::custom("Invalid Unix timestamp"))?;

    Ok(datetime.with_timezone(&FixedOffset::east_opt(0).unwrap()))
}

#[derive(Debug, Clone)]
pub enum SourceKind {
    Source(String),
    Origin,
    Unknown
}

impl Serialize for SourceKind {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            SourceKind::Source(s) => serializer.serialize_str(s),
            SourceKind::Origin => serializer.serialize_str("__special_origin__"),
            SourceKind::Unknown => serializer.serialize_unit(),
        }
    }
}

impl<'de> Deserialize<'de> for SourceKind {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct SourceKindVisitor;

        impl<'de> de::Visitor<'de> for SourceKindVisitor {
            type Value = SourceKind;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a string or null")
            }

            fn visit_str<E>(self, value: &str) -> Result<SourceKind, E>
            where
                E: de::Error,
            {
                match value {
                    "__special_origin__" => Ok(SourceKind::Origin),
                    s => Ok(SourceKind::Source(s.to_owned())),
                }
            }

            fn visit_none<E>(self) -> Result<SourceKind, E>
            where
                E: de::Error,
            {
                Ok(SourceKind::Unknown)
            }

            fn visit_unit<E>(self) -> Result<SourceKind, E>
            where
                E: de::Error,
            {
                Ok(SourceKind::Unknown)
            }
        }

        deserializer.deserialize_any(SourceKindVisitor)
    }
}



/// A unified output format to be displayed on the websocket
#[derive(Serialize, Debug, Clone, Deserialize)]
pub struct UnifyOutput {
    pub id: String,
    pub organisation: String,
    pub title: String,
    pub description: String,
    #[serde(serialize_with = "serialize_dt", deserialize_with = "deserialize_dt")]
    pub time: DateTime<FixedOffset>,
    pub source: SourceKind, // This describes where the content was received from
    pub score: Option<f32>, // Score for importance of the news
}

impl UnifyOutput {
    pub fn to_raw(&self) -> UnifyOutputRaw {
        UnifyOutputRaw {
            data: String::from(serde_json::to_string(self).unwrap()),
        }
    }
}

#[derive(Clone, Debug)]
pub struct UnifyOutputRaw {
    pub data: String,
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
