use std::fmt;
use serde::{Serialize, Deserialize, Deserializer, de};
use serde::de::{Unexpected, Visitor};

#[derive(Serialize, Deserialize, Debug)]
struct Origin {
    #[serde(rename = "@url")]
    url: String,
    #[serde(rename = "#text")]
    publisher: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Item {
    title: String,
    link: String,
    #[serde(rename = "pubDate", deserialize_with = "string_as_rfc2822")]
    publish_date: chrono::DateTime<chrono::offset::FixedOffset>,
    source: Origin

}
#[derive(Serialize, Deserialize, Debug)]
pub struct GoogleMrssResult {
    generator: Option<String>,
    title: Option<String>,
    link: Option<String>,
    language: Option<String>,
    #[serde(rename = "lastBuildDate", deserialize_with = "string_as_rfc2822")]
    last_build_date: chrono::DateTime<chrono::offset::FixedOffset>,
    description: Option<String>,
    #[serde(rename = "item")]
    items: Vec<Item>,
}

fn string_as_rfc2822<'de, D>(deserializer: D) -> Result<chrono::DateTime<chrono::offset::FixedOffset>, D::Error>
where
    D: Deserializer<'de>,
{
    deserializer.deserialize_any(RFC2822Visitor)
}

struct RFC2822Visitor;
impl<'de> Visitor<'de> for RFC2822Visitor {
    type Value = chrono::DateTime<chrono::offset::FixedOffset>;
    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a string representation of a RFC2822 datetime string")
    }
    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        chrono::DateTime::parse_from_rfc2822(value).map_err(|_err| {
            E::invalid_value(Unexpected::Str(value), &"a string representation of a f64")
        })
    }
}