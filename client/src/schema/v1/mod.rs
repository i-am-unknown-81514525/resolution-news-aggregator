use serde::Serialize;
use serde_versioning::Deserialize;
use uuid::Uuid;
use crate::schema::v0::{Coordinate, Size};

#[derive(Deserialize, Serialize, Debug, Clone)]
#[versioning(previous_version = "crate::schema::v0::WindowConfig")]
pub struct WindowConfig {
    pub uuid: Uuid,
    pub coordinate: Coordinate,
    pub size: Size,
    pub search: Option<String>
}

impl WindowConfig {
    pub fn new(coordinate: Coordinate, size: Size, search: Option<&str>) -> Self {
        Self {
            uuid: Uuid::now_v7(),
            coordinate,
            size,
            search: search.map(|s| s.to_string()),
        }
    }
}

impl Default for WindowConfig {
    fn default() -> Self {
        Self {
            uuid: Uuid::nil(),
            coordinate: Coordinate {x: 0, y: 0},
            size: Size {x:200, y:800},
            search: None
        }
    }
}

impl WindowConfig {
    pub fn with_uuid(&mut self) -> Self {
        self.uuid = Uuid::now_v7();
        *self
    }
}