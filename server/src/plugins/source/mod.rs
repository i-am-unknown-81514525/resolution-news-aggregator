mod google_rss_search;
mod hacker_news;

pub(crate) use crate::plugins::net::rss_fetch::RssFetchError;
use crate::plugins::source::google_rss_search::GoogleRssSearch;
use crate::value_enum::{EnumFromStr, value_enum};
use common::unify::{ToVecUnify, UnifyOutput};
use serde::Deserialize;
use serde_xml_rs::from_str;

value_enum!(RSSSourceType, DirectRss, GoogleWrap, GoogleRssSearch);

#[async_trait::async_trait]
pub(crate) trait RSSSource: Send + Sync {
    type Deserialize<'a>: Deserialize<'a> + ToVecUnify + Send + Sync
    where
        Self: 'a;
    fn get_url(&self, value: &str) -> Option<String>;
    fn deserialize(&self, content: &str) -> Result<Self::Deserialize<'_>, RssFetchError> {
        let parsed: Self::Deserialize = from_str(content).map_err(|e| RssFetchError::SerdeXmlParseError(e))?;
        Ok(parsed)
    }

    async fn post_process(&self, content: Vec<UnifyOutput>) -> Vec<UnifyOutput> {
        content
    }
}

pub(crate) fn remap(t: RSSSourceType) -> impl RSSSource {
    match t {
        RSSSourceType::GoogleRssSearch => GoogleRssSearch {},
        _ => todo!(),
    }
}
