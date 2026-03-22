use serde::de::{Unexpected, Visitor};
use serde::{Deserializer, de};
use std::fmt;

#[cfg(feature = "html_handle")]
use html_escape::decode_html_entities_to_string;
#[cfg(feature = "html_handle")]
use html2text::from_read;

pub fn string_as_rfc2822<'de, D>(
    deserializer: D,
) -> Result<chrono::DateTime<chrono::offset::FixedOffset>, D::Error>
where
    D: Deserializer<'de>,
{
    deserializer.deserialize_string(RFC2822Visitor)
}

pub struct RFC2822Visitor;
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
            E::invalid_value(
                Unexpected::Str(value),
                &"a string representation of a RFC2822 datetime string",
            )
        })
    }
}

pub fn string_as_rfc3339<'de, D>(
    deserializer: D,
) -> Result<chrono::DateTime<chrono::offset::FixedOffset>, D::Error>
where
    D: Deserializer<'de>,
{
    deserializer.deserialize_string(RFC3339Visitor)
}

pub struct RFC3339Visitor;
impl<'de> Visitor<'de> for RFC3339Visitor {
    type Value = chrono::DateTime<chrono::offset::FixedOffset>;
    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a string representation of a RFC3339 datetime string")
    }
    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        chrono::DateTime::parse_from_rfc3339(value).map_err(|_err| {
            E::invalid_value(
                Unexpected::Str(value),
                &"a string representation of a RFC3339 datetime string",
            )
        })
    }
}

#[cfg(feature = "html_handle")]
pub fn html_to_text<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    deserializer.deserialize_string(HtmlToTextVisitor)
}

#[cfg(feature = "html_handle")]
pub struct HtmlToTextVisitor;

#[cfg(feature = "html_handle")]
impl<'de> Visitor<'de> for HtmlToTextVisitor {
    type Value = Option<String>;
    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a string representation of escaped/unescaped HTML")
    }
    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        let mut unescaped = String::new();
        decode_html_entities_to_string(value, &mut unescaped);
        let text = from_read(unescaped.as_bytes(), 1_000_000);
        match text {
            Ok(v) => Ok(Some(v)),
            Err(_) => Ok(None)
        }
    }
}
