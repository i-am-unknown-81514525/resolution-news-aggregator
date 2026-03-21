mod google_rss_search;
mod hacker_news;

pub(crate) use crate::plugins::net::rss_fetch::RssFetchError;
use crate::plugins::source::google_rss_search::GoogleRssSearch;
use crate::value_enum::{EnumFromStr, value_enum};
use common::unify::{ToVecUnify, UnifyOutput};
use serde::Deserialize;
use serde_xml_rs::from_str;
use crate::plugins::source::hacker_news::HackerNews;

value_enum!(RSSSourceType, GoogleRssSearch, HackerNews);

#[async_trait::async_trait]
pub(crate) trait RSSSource: Send + Sync {
    type Deserialize<'a>: Deserialize<'a> + ToVecUnify + Send + Sync
    where
        Self: 'a;
    fn get_url(&self, value: &str) -> Option<String>;
    fn deserialize(&self, content: &str) -> Result<Self::Deserialize<'_>, RssFetchError> {
        let parsed: Self::Deserialize<'_> = from_str(content).map_err(|e| RssFetchError::SerdeXmlParseError(e))?;
        Ok(parsed)
    }

    async fn post_process(&self, content: Vec<UnifyOutput>) -> Vec<UnifyOutput> {
        content
    }
}

pub(crate) enum BoxedRSSSource {
    GoogleRssSearch(GoogleRssSearch),
    HackerNews(HackerNews),
}

// Implement the trait for the enum by forwarding calls to the variants
impl BoxedRSSSource {

    pub(crate) fn get_url(&self, value: &str) -> Option<String> {
        match self {
            BoxedRSSSource::GoogleRssSearch(search) => search.get_url(value),
            BoxedRSSSource::HackerNews(search) => search.get_url(value),
        }
    }

    pub(crate) fn get_unify(&self, content: &str) -> Result<Vec<UnifyOutput>, RssFetchError> {
        Ok(match self {
            BoxedRSSSource::GoogleRssSearch(search) => search.deserialize(content)?.to_vec_unify(),
            BoxedRSSSource::HackerNews(search) => search.deserialize(content)?.to_vec_unify(),
        })
    }

    pub(crate) async fn post_process(&self, content: Vec<UnifyOutput>) -> Vec<UnifyOutput> {
        match self {
            BoxedRSSSource::GoogleRssSearch(search) => search.post_process(content).await,
            BoxedRSSSource::HackerNews(search) => search.post_process(content).await,
        }
    }
}

pub(crate) fn remap<'a>(t: RSSSourceType) -> BoxedRSSSource {
    match t {
        RSSSourceType::GoogleRssSearch => BoxedRSSSource::GoogleRssSearch(GoogleRssSearch {}),
        RSSSourceType::HackerNews => BoxedRSSSource::HackerNews(HackerNews {})
    }
}
