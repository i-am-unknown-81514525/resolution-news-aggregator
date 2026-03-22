use crate::plugins::source::RSSSource;

pub(crate) struct Youtube {}

impl RSSSource for Youtube {
    type Deserialize<'a> = crate::plugins::parser::youtube::Outer;

    fn get_url(&self, value: &str) -> Option<String> {
        let query = value.to_string();
        Some(format!(
            "https://www.youtube.com/feeds/videos.xml?channel_id={}",
            query
        ))
    }
}
