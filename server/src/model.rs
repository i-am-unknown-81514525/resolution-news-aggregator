use std::path::PathBuf;
use ahash::{HashMap, HashMapExt, HashSet};
use cfg_if::cfg_if;
#[cfg(feature = "embedding")]
use fastembed::{TextEmbedding, TextInitOptions};
#[cfg(feature = "embedding")]
use fastembed::EmbeddingModel::EmbeddingGemma300M;
use tokio::sync::oneshot;
use common::unify::UnifyOutput;

#[cfg(feature = "embedding")]
pub struct Model(Option<TextEmbedding>);

#[cfg(not(feature = "embedding"))]
pub struct Model;

use std::sync::atomic::{AtomicU64, Ordering};

static ID_COUNTER: AtomicU64 = AtomicU64::new(1);

fn get_unique_id() -> u64 {
    ID_COUNTER.fetch_add(1, Ordering::Relaxed)
}

pub fn get_model() -> Model {
    cfg_if! {
        if #[cfg(feature="embedding")] {
            match TextEmbedding::try_new(
                TextInitOptions::new(EmbeddingGemma300M)
                    .with_cache_dir(PathBuf::from("model_data/"))
                    .with_show_download_progress(true)
            ) {
                Ok(model) => Model(Some(model)),
                Err(e) => Model(None)
            }
        } else {
            Model {}
        }
    }
}

pub struct Task {
    pub uuid: u64,
    pub request: Vec<UnifyOutput>,
    pub response: oneshot::Sender<Vec<Option<Vec<f32>>>>,
}

impl Task {
    pub fn create(data: Vec<UnifyOutput>) -> (Self, oneshot::Receiver<Vec<Option<Vec<f32>>>>) {
        let (tx, rx) = oneshot::channel();
        (
            Self {
                uuid: get_unique_id(),
                request: data,
                response: tx
            },
            rx
        )
    }
}

pub fn embedding_thread(mut model: Model, mut receiver: tokio::sync::mpsc::UnboundedReceiver<Task>) {
    loop {
        let mut tasks = Vec::with_capacity(32);
        if receiver.blocking_recv_many(&mut tasks, 32) == 0 {
            tracing::info!("Sender disconnected, gracefully shutting down embedding thread.");
            return;
        }

        let mut text_to_index: HashMap<String, usize> = HashMap::new();
        let mut texts_to_embed = Vec::new();
        let mut task_item_indices: Vec<Vec<usize>> = Vec::with_capacity(tasks.len());

        cfg_if! {
            if #[cfg(feature="embedding")] {
                for task in &tasks {
                    let indices: Vec<usize> = task.request.iter().map(|item| {
                        let text = format!("{} {}", item.title, item.description);
                        *text_to_index.entry(text).or_insert_with_key(|key| {
                            let index = texts_to_embed.len();
                            texts_to_embed.push(key.clone());
                            index
                        })
                    }).collect();
                    task_item_indices.push(indices);
                }

                if texts_to_embed.is_empty() {
                    for task in tasks {
                        let _ = task.response.send(vec![None; task.request.len()]);
                    }
                    continue;
                }
                if let None = model.0 {
                    for task in tasks {
                        let _ = task.response.send(vec![None; task.request.len()]);
                    }
                    continue;
                }
                let Some(ref mut model_ref) = model.0 else {unreachable!()};
                let embeddings = match model_ref.embed(texts_to_embed, None) {
                    Ok(e) => e,
                    Err(e) => {
                        for task in tasks {
                            let _ = task.response.send(vec![None; task.request.len()]);
                        }
                        continue;
                    }
                };

                for (i, task) in tasks.into_iter().enumerate() {
                    let results: Vec<Option<Vec<f32>>> = task_item_indices[i]
                        .iter()
                        .map(|&index| embeddings.get(index).cloned())
                        .collect();

                    if task.response.send(results).is_err() {
                        tracing::warn!("Failed to send embedding result for task {}", task.uuid);
                    }
                }
            } else {
                for task in tasks {
                    let _ = task.response.send(vec![None; task.request.len()]);
                }
                continue;
            }
        }
    }
}
