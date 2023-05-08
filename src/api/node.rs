use serde::{Deserialize, Serialize};

use std::{default, error::Error, marker::PhantomData};

use crate::api::error::CoggleError;

use super::diagram::CoggleApiDiagram;

// use super::diagram::CoggleApiDiagram;

const MAX_TEXT_LENGTH: usize = 3000;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CoggleApiNode {
    pub id: String,
    pub text: String,
    pub offset: i32,
    pub parent_id: Option<String>,
    pub diagram: CoggleApiDiagram,
    pub children: Vec<CoggleApiNode>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NodeResource {
    #[serde(rename(serialize = "_id"))]
    pub id: String,
    pub text: String,
    pub offset: i32,
    pub parent: Option<String>,
    pub children: Option<Vec<NodeResource>>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct Offset {
    pub x: i32,
    pub y: i32,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct NodeUpdate<'a> {
    pub offset: Option<Offset>,
    pub text: Option<&'a str>,
    pub parent: Option<&'a str>,
}

impl CoggleApiNode {
    pub fn new(coggle_api_diagram: CoggleApiDiagram, node_resource: NodeResource) -> Self {
        let mut node = CoggleApiNode {
            diagram: coggle_api_diagram,
            id: node_resource._id,
            text: node_resource.text,
            offset: node_resource.offset,
            parent_id: node_resource.parent,
            children: Vec::new(),
        };

        if let Some(children) = node_resource.children {
            for child_resource in children {
                let mut child = CoggleApiNode::new(coggle_api_diagram, child_resource);
                child.parent_id = Some(node.id.clone());
                node.children.push(child);
            }
        }

        node
    }

    pub fn replace_ids(&self, url: &str) -> String {
        self.diagram.replace_id(&url.replace(":node", &self.id))
    }

    pub async fn add_child<'a>(
        &self,
        text: &str,
        offset: &Offset,
    ) -> Result<CoggleApiNode<'a>, impl Error> {
        if text.len() > MAX_TEXT_LENGTH {
            return Err(CoggleError::TextTooLong.into());
        }
        #[derive(Serialize)]
        struct Body<'a> {
            parent: &'a str,
            offset: Option<&'a CoggleOffset>,
            text: &'a str,
        }
        let body = json!({
            parent: self.id,
            offset,
            text,
        });

        self.diagram
            .api_client
            .post(
                &self.replace_ids("/api/1/diagrams/:diagram/nodes"),
                "",
                body,
            )
            .await
            .map(|node_resource| {
                let api_node = CoggleApiNode::new(self.diagram, node_resource);
                api_node.parent_id = Some(self.id);
                self.children.push(api_node);
                api_node
            })
    }

    // TODO update

    pub async fn set_text(self, text: &str) {
        self.update(NodeUpdate {
            text: Some(text),
            ..Default::default()
        })
    }

    pub async fn r#move(self, offset: &Offset) {
        self.update(NodeUpdate {
            offset: Some(offset),
            ..Default::default()
        })
    }
}
