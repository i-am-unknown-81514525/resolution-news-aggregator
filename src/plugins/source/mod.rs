use std::collections::HashMap;
use crate::value_enum::{EnumFromStr, value_enum};
use once_cell::sync::Lazy;

value_enum!(RSSSourceType, DirectRss, GoogleWrap, GoogleRssSearch);

pub(crate) trait RSSSource {
    fn get_url(&self, rss_type: RSSSourceType, value: &str) -> Option<String>;
}

static RSS_SOURCE_TYPE_MAPPING: Lazy<HashMap<RSSSourceType, Box<dyn RSSSource + Send + Sync>>> = Lazy::new(
    || HashMap::from(
        [

        ]
    )
);