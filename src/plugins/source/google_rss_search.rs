// https://news.google.com/rss/search?q=(oil+price+OR+OPEC+OR+"natural+gas"+OR+"crude+oil"+OR+WTI+OR+Brent)+when:1d&hl=en-US&gl=US&ceid=US:en

use serde_xml_rs::from_str;
use crate::plugins::rss_fetch::RssFetchError;
use crate::plugins::source::{RSSSource, RSSSourceType};
use crate::plugins::parser::google_mrss::GoogleMrssResult;

pub(crate) struct GoogleRssSearch {}

impl RSSSource for GoogleRssSearch {
    type Deserialize = GoogleMrssResult;

    fn get_url(&self, value: &str) -> Option<String> {
        let query = value.to_string();
        return Some(format!("https://news.google.com/rss/search?q=({})&hl=en-US&gl=US&ceid=US:en", query))
    }

    fn deserialize(&self, content: &str) -> Result<Self::Deserialize, RssFetchError> {
        let parsed: GoogleMrssResult = from_str(content).map_err(|e| RssFetchError::SerdeXmlParseError(e))?;
        Ok(parsed)
    }
}