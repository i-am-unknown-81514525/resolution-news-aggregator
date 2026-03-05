use chrono::{DateTime,offset::Utc};
use async_trait::async_trait;


/// A unified output format to be displayed on the websocket
pub struct UnifyOutput {
    organisation: String,
    title: String,
    description: String,
    time: DateTime<Utc>,
    score: Option<f32> // Score for importance of the news
}


pub trait Config {}

#[async_trait]
pub trait NewsScraper<T: Config> {
    fn from(config: T) -> Box<Self>;

    async fn retrieve(self) -> Vec<UnifyOutput>;
}