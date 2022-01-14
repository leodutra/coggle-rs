use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CoggleOffset {
    pub x: i32,
    pub y: i32,
}
