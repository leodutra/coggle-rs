// use http::HTTP_CLIENT;

use serde::{Deserialize, Serialize};
use std::error::Error;

use super::{diagram::CoggleApiDiagram, error::CoggleError};

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
            parent: node.parent,
            text: node.text.into(),
            offset: node.offset.into(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CoggleOffset {
    pub x: i32,
    pub y: i32,
}

#[derive(Serialize, Debug, Clone)]
pub struct CoggleApiNode<'a> {
    pub diagram: &'a CoggleApiDiagram<'a>,

    #[serde(rename(serialize = "_id"))]
    pub id: String,             // unique ID of the node
    pub text: String,           // text of node
    pub offset: CoggleOffset,   // offset from parent node
    pub text_size: Option<f32>, // size of the text node
    pub width: Option<f32>,     // width of node
    pub colour: Option<String>, // colour of node
    pub parent: Option<String>, // parent node id
    pub children: Option<Vec<CoggleApiNode<'a>>>,
}

impl<'a> CoggleApiNode<'a> {
    pub fn new(
        coggle_api_diagram: &'a CoggleApiDiagram,
        node_resource: &CoggleNodeResource,
    ) -> Self {
        CoggleApiNode {
            diagram: coggle_api_diagram,
            id: node_resource.id.clone(),
            text: node_resource.text.clone(),
            offset: node_resource.offset.clone(),
            children: Some(
                node_resource
                    .children
                    .iter()
                    .map(|child_resource| CoggleApiNode::new(coggle_api_diagram, child_resource))
                    .collect(),
            ),
            parent: node_resource.parent.clone(),
            text_size: None,
            width: None,
            colour: None,
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
            .unwrap()
            .post(
                &self.replace_ids("/api/1/diagrams/:diagram/nodes"),
                "",
                &body,
            )
            .await?
            .json()
            .await?;
        let mut api_node = CoggleApiNode::new(self.diagram, &node_resource);
        api_node.parent = Some(self.id.clone());
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
            .unwrap()
            .post(
                &self.replace_ids("/api/1/diagrams/:diagram/nodes"),
                "",
                &properties,
            )
            .await?
            .json()
            .await?;
        let mut api_node = CoggleApiNode::new(self.diagram, &node_resource);
        api_node.parent = Some(self.id.clone());
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
