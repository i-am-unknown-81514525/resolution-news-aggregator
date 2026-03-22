use crate::plugins::source::RSSSource;

pub(crate) struct Reddit {}

impl RSSSource for Reddit {
    type Deserialize<'a> = crate::plugins::parser::reddit::Outer;

    fn get_url(&self, value: &str) -> Option<String> {
        let query = value.to_string();
        Some(format!(
            "https://www.reddit.com/r/{}/.rss",
            query
        ))
    }
}
