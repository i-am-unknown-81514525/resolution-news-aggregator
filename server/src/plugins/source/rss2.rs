use crate::plugins::source::RSSSource;

pub(crate) struct Rss2Generic {}

impl RSSSource for Rss2Generic {
    type Deserialize<'a> = crate::plugins::parser::rss2::Outer;

    fn get_url(&self, value: &str) -> Option<String> {
        let query = value.to_string();
        Some(query.to_string())
    }
}
