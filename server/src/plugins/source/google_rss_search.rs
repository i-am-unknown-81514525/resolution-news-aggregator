// https://news.google.com/rss/search?q=(oil+price+OR+OPEC+OR+"natural+gas"+OR+"crude+oil"+OR+WTI+OR+Brent)+when:1d&hl=en-US&gl=US&ceid=US:en

use crate::plugins::source::RSSSource;

pub(crate) struct GoogleRssSearch {}

impl RSSSource for GoogleRssSearch {
    type Deserialize<'a> = crate::plugins::parser::google_mrss::Outer;

    fn get_url(&self, value: &str) -> Option<String> {
        let query = value.to_string();
        Some(format!(
            "https://news.google.com/rss/search?q={}&hl=en-US&gl=US&ceid=US:en",
            query
        ))
    }
}
