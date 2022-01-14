
use regex::Regex;
use reqwest::Response;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::{
    error::Error,
    fmt::{self, Display, Formatter},
};

use super::{node::{CoggleApiNode, CoggleNodeResource}, CoggleApi};
    


pub struct CoggleApiDiagram<'a> {
    pub api_client: &'a CoggleApi,
    pub id: String,
    pub title: String,
}

impl<'a> CoggleApiDiagram<'a> {
    pub fn new(coggle_api: &'a CoggleApi, diagram_resource: &CoggleDiagramResource) -> Self {
        CoggleApiDiagram {
            api_client: coggle_api,
            id: diagram_resource.id.clone(),
            title: diagram_resource.title.clone(),
        }
    }

    pub fn replace_id(&self, url: &str) -> String {
        url.replacen(":diagram", &self.id, 1)
    }

    pub fn web_url(&self) -> String {
        self.replace_id(&(self.api_client.base_url.clone() + "/diagram/:diagram"))
    }

    pub async fn get_nodes(&self) -> Result<Vec<CoggleApiNode<'_>>, Box<dyn Error>> {
        let node_resources: Vec<CoggleNodeResource> = self
            .api_client
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