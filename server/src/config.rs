use serde::Deserialize;

#[derive(Deserialize)]
pub struct Config {
    #[serde(rename = "type")]
    pub(crate) rss_type: String,
    pub(crate) query: String,
    pub(crate) update_interval: u16
}

#[derive(Deserialize)]
pub struct Configs {
    #[serde(deserialize_with = "warn_incorrect", rename = "config")]
    pub(crate) configs: Vec<Config>
}

fn warn_incorrect<'de, D, T>(deserializer: D) -> Result<Vec<T>, D::Error>
where
    D: serde::Deserializer<'de>,
    T: serde::Deserialize<'de>,
{
    // 1. Parse the whole list into generic JSON Values
    let values: Vec<serde_json::Value> = serde::Deserialize::deserialize(deserializer)?;

    let span = tracing::warn_span!("Parsing configs");
    span.in_scope(|| Ok(values.into_iter().filter_map(|v| {
        match T::deserialize(v) {
            Ok(item) => Some(item),
            Err(e) => {
                tracing::warn!("Unable to parse content: {}", e);
                None
            }
        }
    }).collect()))

}