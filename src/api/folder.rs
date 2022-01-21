use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct CoggleFolder {
    pub id: String,
    pub name: String,
    pub folder: Box<CoggleFolder>, // (readonly) Folders can be nested. Note: reserved for future use
    pub created_by: String,
    pub my_access: Vec<String>, // Array of permissions on the folder for the current user
}
