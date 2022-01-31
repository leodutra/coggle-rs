use serde::{Deserialize, Serialize};
use std::error::Error;

use super::{
    folder::CoggleFolder,
    node::{CoggleApiNode, CoggleNodeResource},
    CoggleApi,
};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CoggleApiDiagram<'a> {
    #[serde(skip)]
    pub api_client: Option<&'a CoggleApi>,

    pub id: String,
    pub title: String,
    pub timestamp: Option<String>,
    pub my_access: Option<Vec<String>>,
    pub owner_id: Option<String>,
    pub modified: Option<String>,
    pub folder: Option<CoggleFolder>,
}

impl<'a> CoggleApiDiagram<'a> {
    pub fn new(coggle_api: &'a CoggleApi, diagram_resource: &CoggleDiagramResource) -> Self {
        CoggleApiDiagram {
            api_client: Some(coggle_api),
            id: diagram_resource.id.clone(),
            title: diagram_resource.title.clone(),
            timestamp: None,
            my_access: None,
            owner_id: None,
            modified: None,
            folder: None,
        }
    }

    pub fn replace_id(&self, url: &str) -> String {
        url.replacen(":diagram", &self.id, 1)
    }

    pub fn web_url(&self) -> String {
        self.replace_id(&(self.api_client.unwrap().base_url.clone() + "/diagram/:diagram"))
    }

    pub async fn get_nodes(&self) -> Result<Vec<CoggleApiNode<'_>>, Box<dyn Error>> {
        let node_resources: Vec<CoggleNodeResource> = self
            .api_client
            .unwrap()
            .get(&self.replace_id("/api/1/diagrams/:diagram/nodes"), "")
            .await?;

        let nodes = node_resources
            .iter()
            .map(|node_resource| CoggleApiNode::new(self, node_resource))
            .collect();
        Ok(nodes)
    }

    pub async fn arrange(&self) -> Result<Vec<CoggleApiNode<'_>>, Box<dyn Error>> {
        #[derive(Serialize)]
        struct Body {}
        let node_resources: Vec<CoggleNodeResource> = self
            .api_client
            .unwrap()
            .put(
                &self.replace_id("/api/1/diagrams/:diagram/nodes"),
                "action=arrange",
                &Body {},
            )
            .await?;

        let nodes = node_resources
            .iter()
            .map(|node_resource| CoggleApiNode::new(self, node_resource))
            .collect();
        Ok(nodes)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CoggleDiagramResource {
    #[serde(rename(serialize = "_id"))]
    id: String,
    title: String,
}
