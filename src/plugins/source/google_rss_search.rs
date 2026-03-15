// https://news.google.com/rss/search?q=(oil+price+OR+OPEC+OR+"natural+gas"+OR+"crude+oil"+OR+WTI+OR+Brent)+when:1d&hl=en-US&gl=US&ceid=US:en

use crate::plugins::rss_fetch::RssFetchError;
use crate::plugins::source::{RSSSource, RSSSourceType};

pub(crate) struct GoogleRssSearch {}

impl RSSSource for GoogleRssSearch {
    fn get_url(&self, value: &str) -> Option<String> {
        let query = value.to_string();
        return Some(format!("https://news.google.com/rss/search?q=({})&hl=en-US&gl=US&ceid=US:en", query))
    }

    fn deserialize(&self, content: &String) -> Result<Self::Deserialize, RssFetchError> {
        
    }
}