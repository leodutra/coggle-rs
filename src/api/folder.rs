pub struct Folder {
    pub id: String,
    pub name: String,
    pub folder: Vec<Folder>,
    pub created_at: String,
    pub my_access: String,
}

impl Folder {
    pub fn new(folder_resource: FolderResource) -> Self {
        let mut folder = Folder {
            id: folder_resource._id,
            name: folder_resource.name,
            folder: Vec::new(),
            created_at: folder_resource.created_at,
            my_access: folder_resource.my_access,
        };

        if let Some(children) = folder_resource.children {
            for child_resource in children {
                let child = Folder::new(child_resource);
                folder.folder.push(child);
            }
        }

        folder
    }

    // pub fn replace_ids(&self, url: &str) -> String {
    //     url.replace(":folder", &self.id)
    // }
    
}