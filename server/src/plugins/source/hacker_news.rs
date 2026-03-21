use crate::plugins::source::RSSSource;

pub(crate) struct HackerNews {}

impl RSSSource for HackerNews {
    type Deserialize<'a> = crate::plugins::parser::hacker_news::Outer;

    fn get_url(&self, value: &str) -> Option<String> {
        let _query = value.to_string();
        return Some("https://news.ycombinator.com/rss".to_string());
    }
}
