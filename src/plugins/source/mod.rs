mod google_rss_search;

use std::collections::HashMap;
use crate::value_enum::{EnumFromStr, value_enum};
use once_cell::sync::Lazy;
use serde::Deserialize;
use crate::plugins::source::google_rss_search::GoogleRssSearch;
use crate::plugins::net::rss_fetch::RssFetchError;
use crate::unify::ToVecUnify;

value_enum!(RSSSourceType, DirectRss, GoogleWrap, GoogleRssSearch);

pub(crate) trait RSSSource : Send + Sync {
    type Deserialize<'a>: Deserialize<'a> + ToVecUnify where Self: 'a;
    fn get_url(&self, value: &str) -> Option<String>;
    fn deserialize(&self, content: &str) -> Result<Self::Deserialize<'_>, RssFetchError>;
}

pub(crate) fn remap(t: RSSSourceType) -> impl RSSSource {
    match t {
        RSSSourceType::GoogleRssSearch => GoogleRssSearch {},
        _ => todo!()
    }
}