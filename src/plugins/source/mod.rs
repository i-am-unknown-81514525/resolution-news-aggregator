mod google_rss_search;

use std::collections::HashMap;
use crate::value_enum::{EnumFromStr, value_enum};
use once_cell::sync::Lazy;
use crate::plugins::source::google_rss_search::GoogleRssSearch;

value_enum!(RSSSourceType, DirectRss, GoogleWrap, GoogleRssSearch);

pub(crate) trait RSSSource : Send + Sync {
    fn get_url(&self, value: &str) -> Option<String>;
}

pub(crate) fn remap(t: RSSSourceType) -> impl RSSSource {
    match t {
        RSSSourceType::GoogleRssSearch => GoogleRssSearch {},
        _ => todo!()
    }
}