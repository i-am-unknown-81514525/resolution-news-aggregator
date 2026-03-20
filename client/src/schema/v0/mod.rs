use std::fmt::Display;
use serde::Serialize;
use serde_versioning::Deserialize;
use uuid::Uuid;

#[derive(Clone, Serialize, Deserialize, Debug, Eq, PartialEq, Hash)]
pub struct Coordinate {
    pub x: u32,
    pub y: u32,
}

impl Display for Coordinate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}

#[derive(Clone, Serialize, Deserialize, Debug, Eq, PartialEq, Hash)]
pub struct Size {
    pub x: u32,
    pub y: u32,
}

impl Display for Size {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct WindowConfig {
    pub uuid: Uuid,
    pub coordinate: Coordinate,
    pub size: Size
}

impl WindowConfig {
    pub fn new(coordinate: Coordinate, size: Size) -> Self {
        Self {
            uuid: Uuid::now_v7(),
            coordinate,
            size
        }
    }
}

impl Default for WindowConfig {
    fn default() -> Self {
        Self {
            uuid: Uuid::nil(),
            coordinate: Coordinate {x: 0, y: 0},
            size: Size {x:200, y:800}
        }
    }
}

impl WindowConfig {
    pub fn with_uuid(mut self) -> Self {
        self.uuid = Uuid::now_v7();
        self
    }
}