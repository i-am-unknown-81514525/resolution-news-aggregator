use std::path::PathBuf;
use cfg_if::cfg_if;
#[cfg(feature = "embedding")]
use fastembed::{TextEmbedding, TextInitOptions};
#[cfg(feature = "embedding")]
use fastembed::EmbeddingModel::EmbeddingGemma300M;

#[cfg(feature = "embedding")]
pub struct Model(Option<TextEmbedding>);

#[cfg(not(feature = "embedding"))]
pub struct EmbeddingModel;


pub fn get_model() -> Model {
    cfg_if! {
        if #[cfg(feature="embedding")] {
            match TextEmbedding::try_new(
                TextInitOptions::new(EmbeddingGemma300M)
                    .with_cache_dir(PathBuf::from("/model_data"))
                    .with_show_download_progress(true)
            ) {
                Ok(model) => Model(Some(model)),
                Err(e) => Model(None)
            }
        } else {
            model = Model {};
        }
    }
}