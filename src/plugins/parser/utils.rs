use std::fmt;
use serde::de::{Unexpected, Visitor};
use serde::{de, Deserializer};

pub fn string_as_rfc2822<'de, D>(deserializer: D) -> Result<chrono::DateTime<chrono::offset::FixedOffset>, D::Error>
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
            E::invalid_value(Unexpected::Str(value), &"a string representation of a RFC2822 datetime string")
        })
    }
}