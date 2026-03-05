use chrono::{DateTime,offset::Utc};


/// A unified output format to be displayed on the websocket
pub struct UnifyOutput {
    organisation: String,
    title: String,
    description: String,
    time: DateTime<Utc>,
    score: Option<f32> // Score for importance of the news
}