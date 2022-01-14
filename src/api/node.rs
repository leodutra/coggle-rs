
// use http::HTTP_CLIENT;
use regex::Regex;
use reqwest::Response;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::{
    error::Error,
    fmt::{self, Display, Formatter},
};

use super::{error::CoggleError, misc::CoggleOffset, diagram::CoggleApiDiagram};

const MAX_TEXT_LENGTH: usize = 3000;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CoggleNodeResource {
    #[serde(rename(serialize = "_id"))]
    id: String,
    text: String,
    offset: CoggleOffset,
    parent: Option<String>,
    children: Vec<CoggleNodeResource>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct CoggleNodeUpdateProps {
    pub text: Option<String>,
    pub offset: Option<CoggleOffset>,
    pub parent: Option<String>,
}


impl From<CoggleApiNode<'_>> for CoggleNodeUpdateProps {
    fn from(node: CoggleApiNode) -> Self {
        CoggleNodeUpdateProps {
            parent: node.parent_id.into(),
            text: node.text.into(),
            offset: node.offset.into(),
        }
    }
}

pub struct CoggleApiNode<'a> {
    pub diagram: &'a CoggleApiDiagram<'a>,
    pub id: String,
    pub text: String,
    pub offset: CoggleOffset,
    pub children: Vec<CoggleApiNode<'a>>,
    pub parent_id: Option<String>,
}

impl<'a> CoggleApiNode<'a> {
    pub fn new(coggle_api_diagram: &'a CoggleApiDiagram, node_resource: &CoggleNodeResource) -> Self {
        CoggleApiNode {
            diagram: coggle_api_diagram,
            id: node_resource.id.clone(),
            text: node_resource.text.clone(),
            offset: node_resource.offset.clone(),
            children: node_resource
                .children
                .iter()
                .map(|child_resource| CoggleApiNode::new(coggle_api_diagram, child_resource))
                .collect(),
            parent_id: node_resource.parent.clone(),
        }
    }

    pub fn replace_ids(&self, url: &str) -> String {
        self.diagram.replace_id(&url.replacen(":node", &self.id, 1))
    }

    pub async fn add_child(
        &self,
        text: &str,
        offset: Option<&CoggleOffset>,
    ) -> Result<CoggleApiNode<'_>, Box<dyn Error>> {
        if text.len() > MAX_TEXT_LENGTH {
            return Err(CoggleError::TextTooLong.into());
        }

        #[derive(Serialize)]
        struct Body<'a> {
            parent: &'a str,
            offset: Option<&'a CoggleOffset>,
            text: &'a str,
        }
        let body = Body {
            parent: &self.id,
            offset,
            text,
        };
        let node_resource = self
            .diagram
            .api_client
            .post(
                &self.replace_ids("/api/1/diagrams/:diagram/nodes"),
                "",
                &body,
            )
            .await?
            .json()
            .await?;
        let mut api_node = CoggleApiNode::new(self.diagram, &node_resource);
        api_node.parent_id = Some(self.id.clone());
        Ok(api_node)
    }

    pub async fn update(
        &self,
        properties: CoggleNodeUpdateProps,
    ) -> Result<CoggleApiNode<'_>, Box<dyn Error>> {
        if let Some(text) = &properties.text {
            if text.len() > MAX_TEXT_LENGTH {
                return Err(CoggleError::TextTooLong.into());
            }
        }

        let node_resource = self
            .diagram
            .api_client
            .post(
                &self.replace_ids("/api/1/diagrams/:diagram/nodes"),
                "",
                &properties,
            )
            .await?
            .json()
            .await?;
        let mut api_node = CoggleApiNode::new(self.diagram, &node_resource);
        api_node.parent_id = Some(self.id.clone());
        Ok(api_node)
    }

    pub async fn set_text(&self, text: &str) -> Result<CoggleApiNode<'_>, Box<dyn Error>> {
        self.update(CoggleNodeUpdateProps {
            text: Some(text.to_owned()),
            ..Default::default()
        })
        .await
    }

    pub async fn r#move(&self, offset: &CoggleOffset) -> Result<CoggleApiNode<'_>, Box<dyn Error>> {
        self.update(CoggleNodeUpdateProps {
            offset: Some(offset.clone()),
            ..Default::default()
        })
        .await
    }

    // pub async fn remove(&self) -> Result<CoggleApiNode<'_>, Box<dyn Error>> {
    //     self.diagram
    //         .api_client
    //         .delete(
    //             &self.replace_ids("/api/1/diagrams/:diagram/nodes/:node"),
    //             "",
    //         )
    //         .await
    // }
}