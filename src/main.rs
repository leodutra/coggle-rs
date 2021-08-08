#[macro_use]
extern crate lazy_static;

mod http;

use http::HTTP_CLIENT;
use regex::Regex;
use reqwest::Response;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::{
    error::Error,
    fmt::{self, Display, Formatter},
};

const MAX_TEXT_LENGTH: usize = 3000;

lazy_static! {
    static ref ORG_NAME_VALIDATION: Regex =
        Regex::new(r"^[a-z]+[a-z0-9-]{2,}$").expect("Organization name validator.");
}

#[derive(Debug)]
// #[non_exhaustive]
pub enum CoggleError {
    TextTooLong,
    InvalidOrganizationName,
}

impl Error for CoggleError {}

impl Display for CoggleError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        use CoggleError::{InvalidOrganizationName, TextTooLong};
        match self {
            TextTooLong => write!(f, "Error: the text is too long."),
            InvalidOrganizationName => write!(f, "Error: invalid organization name."),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CoggleOffset {
    pub x: i32,
    pub y: i32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct CoggleDiagramResource {
    #[serde(rename(serialize = "_id"))]
    id: String,
    title: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct CoggleNodeResource {
    #[serde(rename(serialize = "_id"))]
    id: String,
    text: String,
    offset: CoggleOffset,
    parent: Option<String>,
    children: Vec<CoggleNodeResource>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct NodeUpdateProps {
    pub text: Option<String>,
    pub offset: Option<CoggleOffset>,
    pub parent: Option<String>,
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
    fn new(coggle_api_diagram: &'a CoggleApiDiagram, node_resource: &CoggleNodeResource) -> Self {
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
            parent: &*self.id,
            offset,
            text: &*text,
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
        properties: NodeUpdateProps,
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
        self.update(NodeUpdateProps {
            text: Some(text.to_owned()),
            ..Default::default()
        })
        .await
    }

    pub async fn r#move(&self, offset: &CoggleOffset) -> Result<CoggleApiNode<'_>, Box<dyn Error>> {
        self.update(NodeUpdateProps {
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

pub struct CoggleApiDiagram<'a> {
    pub api_client: &'a CoggleApi,
    pub id: String,
    pub title: String,
}

impl<'a> CoggleApiDiagram<'a> {
    fn new(coggle_api: &'a CoggleApi, diagram_resource: &CoggleDiagramResource) -> Self {
        CoggleApiDiagram {
            api_client: coggle_api,
            id: diagram_resource.id.clone(),
            title: diagram_resource.title.clone(),
        }
    }

    pub fn replace_id(&self, url: &str) -> String {
        // ok
        url.replacen(":diagram", &self.id, 1)
    }

    pub fn web_url(&self) -> String {
        self.replace_id(&(self.api_client.base_url.clone() + "/diagram/:diagram"))
        // ok
    }

    pub async fn get_nodes(&self) -> Result<Vec<CoggleApiNode<'_>>, impl Error> {
        let result = self
            .api_client
            .get(&self.replace_id("/api/1/diagrams/:diagram/nodes"), "")
            .await;

        result.map(|node_resources: Vec<CoggleNodeResource>| {
            node_resources
                .iter()
                .map(|node_resource| CoggleApiNode::new(self, node_resource))
                .collect()
        })
    }
}

pub struct CoggleApi {
    pub base_url: String,
    pub token: String,
}

impl CoggleApi {
    // FIXME: querystring
    pub async fn post<T: Serialize>(
        &self,
        endpoint: &str,
        query_string: &str,
        body: &T,
    ) -> Result<Response, Box<dyn Error>> {
        let prefixed_query = if query_string.starts_with('&') {
            query_string.to_owned()
        } else {
            '&'.to_string() + query_string
        };
        HTTP_CLIENT
            .post(
                self.base_url.to_owned()
                    + endpoint
                    + "?access_token="
                    + &self.token
                    + &prefixed_query,
            )
            .json(&body)
            .send()
            .await
            .map_err(|e| Box::new(e) as Box<dyn Error>)
    }

    // FIXME: querystring
    pub async fn put<'de, T: Serialize + DeserializeOwned>(
        &self,
        endpoint: &str,
        query_string: &str,
        body: &T,
    ) -> Result<T, impl Error> {
        let prefixed_query = if query_string.starts_with('&') {
            query_string.to_owned()
        } else {
            '&'.to_string() + query_string
        };
        HTTP_CLIENT
            .put(
                self.base_url.to_owned()
                    + endpoint
                    + "?access_token="
                    + &self.token
                    + &prefixed_query,
            )
            .json(&body)
            .send()
            .await?
            .json()
            .await
    }

    // FIXME: querystring
    pub async fn get<T: DeserializeOwned>(
        &self,
        endpoint: &str,
        query_string: &str,
    ) -> Result<T, impl Error> {
        let prefixed_query = if query_string.starts_with('&') {
            query_string.to_owned()
        } else {
            '&'.to_string() + query_string
        };
        HTTP_CLIENT
            .get(
                self.base_url.to_owned()
                    + endpoint
                    + "?access_token="
                    + &self.token
                    + &prefixed_query,
            )
            .send()
            .await?
            .json()
            .await
    }

    // FIXME: querystring
    pub async fn delete<T: DeserializeOwned>(
        &self,
        endpoint: &str,
        query_string: &str,
    ) -> Result<T, Box<dyn Error>> {
        let prefixed_query = if query_string.starts_with('&') {
            query_string.to_owned()
        } else {
            '&'.to_string() + query_string
        };
        let result = HTTP_CLIENT
            .delete(
                self.base_url.to_owned()
                    + endpoint
                    + "?access_token="
                    + &self.token
                    + &prefixed_query,
            )
            .send()
            .await?
            .json()
            .await?;
        Ok(result)
    }

    pub async fn list_diagrams(
        &self,
        organization: Option<&str>,
    ) -> Result<Vec<CoggleApiDiagram<'_>>, Box<dyn Error>> {
        if let Some(org) = organization {
            if !ORG_NAME_VALIDATION.is_match(org) {
                return Err(CoggleError::InvalidOrganizationName.into());
            }
        }
        let diagram_resources: Vec<CoggleDiagramResource> = if let Some(organization) = organization
        {
            self.get(
                &format!("/api/1/organisations/{}/diagrams", organization),
                "",
            )
            .await?
        } else {
            self.get("/api/1/diagrams", "").await?
        };
        Ok(diagram_resources
            .iter()
            .map(|resource| CoggleApiDiagram::new(self, resource))
            .collect())
    }
}

fn main() {
    println!("Hello, world!");
}
