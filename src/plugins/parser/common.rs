use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct DocumentID {
    #[serde(rename = "isPermaLink")]
    pub is_perma: Option<bool>,
    #[serde(rename = "#text")]
    pub id: String,
}