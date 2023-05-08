use serde::{Deserialize, Serialize};
use std::error::Error;

use super::{ node::CoggleApiNode, CoggleApi};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CoggleApiDiagram {
    // #[serde(borrow)]
    pub api_client: CoggleApi,
    pub id: String,
    pub title: String,
}

pub struct  DiagramResource {
    pub _id: String,
    pub title: String,
}

impl CoggleApiDiagram {
    pub fn new(coggle_api: &CoggleApi, diagram_resource: DiagramResource) -> Self {
        CoggleApiDiagram {
            api_client: coggle_api,
            id: diagram_resource._id,
            title: diagram_resource.title,
        }
    }

    pub fn web_url(&self) -> String {
        self.replace_id(&format!("{}/diagram/:diagram", self.api_client.base_url))
    }

    pub fn replace_id(&self, url: &str) -> String {
        url.replace(":diagram", &self.id)
    }

    pub async fn get_nodes(&self) -> Result<Vec<CoggleApiNode>, impl Error> {
        self.api_client
            .get(self.replace_id("/api/1/diagrams/:diagram/nodes"))
            .await?
            .map(|body| {
                let mut api_nodes = Vec::new();
                body.forEach(|node_resource| {
                    api_nodes.push(CoggleApiNode::new(*self, node_resource));
                });
                api_nodes
            })
    }

    pub async fn arrange(&self) -> Result<Vec<CoggleApiNode>, impl Error> {
        self.api_client
            .put(
                self.replace_id("/api/1/diagrams/:diagram/nodes"),
                "",
                json!({}),
            )
            .await?
            .map(|body| {
                let mut api_nodes = Vec::new();
                body.forEach(|node_resource| {
                    api_nodes.push(CoggleApiNode::new(self, node_resource));
                });
                api_nodes
            })
    }
}
