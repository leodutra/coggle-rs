#[macro_use]
extern crate lazy_static;

mod http;

use http::HTTP_CLIENT;
use std::{error::Error, fmt::{self, Display, Formatter}};
use reqwest::Response;
use serde::{Serialize, de::DeserializeOwned, Deserialize};

const MAX_TEXT_LENGTH: usize = 3000;

#[derive(Debug)]
// #[non_exhaustive]
pub enum CoggleError {
    TextTooLong
}

impl Error for CoggleError {}

impl Display for CoggleError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            CoggleError::TextTooLong => {
                write!(f, "Error: the text is too long.")
            }
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CoggleOffset {
    pub x: i32,
    pub y: i32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct CoggleNodeResource {
    #[serde(rename(serialize = "_id"))]
    pub id: String,
    pub text: String,
    pub offset: CoggleOffset,
    pub parent: Option<String>,
    pub children: Vec<CoggleNodeResource>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
struct NodeUpdateProps {
    text: Option<String>,
    offset: Option<CoggleOffset>,
    parent: Option<String>,
}

pub struct CoggleApiNode<'a> {
    pub diagram: &'a CoggleApiDiagram<'a>,
    pub id: String,
    pub text: String,
    pub offset: CoggleOffset,
    pub children: Vec<CoggleApiNode<'a>>,
    pub parent_id: Option<String>,
}

impl <'a> CoggleApiNode<'a> {
    pub fn new(coggle_api_diagram: &'a CoggleApiDiagram, node_resource: &CoggleNodeResource) -> Self {
        CoggleApiNode {
            diagram: coggle_api_diagram,
            id: node_resource.id,
            text: node_resource.text,
            offset: node_resource.offset,
            children: node_resource
                .children
                .iter()
                .map(|child_resource| CoggleApiNode::new(coggle_api_diagram, child_resource))
                .collect(),
            parent_id: node_resource.parent,
        }
    }

    pub fn replace_ids(&self, url: &str) -> String {
        self.diagram.replace_id(&url.replacen(":node", &self.id, 1))
    }

    pub async fn add_child(&self, text: &str, offset: Option<&CoggleOffset>) -> Result<CoggleApiNode<'_>, Box<dyn Error>>{
        if text.len() > MAX_TEXT_LENGTH {
            return Err(Box::new(CoggleError::TextTooLong));
        }

        #[derive(Serialize)]
        struct Body {
            parent: String,
            offset: Option<CoggleOffset>,
            text: String,
        }
        let body = Body {
            parent: self.id.clone(),
            offset: offset.map(|x| *x),
            text: text.to_owned(),
        };
        let node_resource = self.diagram.api_client.post(
            &self.replace_ids("/api/1/diagrams/:diagram/nodes"),
            "",
            &body
        ).await?
        .json()
        .await?;
        let mut api_node = CoggleApiNode::new(self.diagram, &node_resource);
        api_node.parent_id = Some(self.id.clone());
        Ok(api_node)
    }

    pub async fn update(&self, properties: NodeUpdateProps) -> Result<CoggleApiNode<'_>, Box<dyn Error>> {
        if let Some(text) = &properties.text {
            if text.len() > MAX_TEXT_LENGTH {
                return Err(Box::new(CoggleError::TextTooLong));
            }
        }

        // #[derive(Serialize)]
        // struct Body {
        //     parent: String,
        //     offset: Option<CoggleOffset>,
        //     text: String,
        // }
        // let body = Body {
        //     parent: self.id,
        //     offset: properties.offset,
        //     text: properties.text.to_owned(),
        // };
        let node_resource = self.diagram.api_client.post(
            &self.replace_ids("/api/1/diagrams/:diagram/nodes"),
            "",
            &properties
        ).await?
        .json()
        .await?;
        let mut api_node = CoggleApiNode::new(self.diagram, &node_resource);
        api_node.parent_id = Some(self.id);
        Ok(api_node)
    }

    pub async fn set_text(&self, text: &str) {
        self.update(CoggleNodeResourcePartial { 
            text: Some(text.to_owned()),
            ..Default::default()
        })
    }

    pub async fn r#move(&self, offset: &CoggleOffset) {
        self.update(CoggleNodeResourcePartial { 
            offset: Some(offset.clone()),
            ..Default::default()
        })
    }

    pub async fn remove(&self) {
        self.diagram.api_client.delete(
            self.replace_ids("/api/1/diagrams/:diagram/nodes/:node"),
            ""
        )
    } 
}

pub struct CoggleApiDiagram<'a> {
    pub api_client: &'a CoggleApi,
    pub id: String,
    pub title: String,
}

impl CoggleApiDiagram<'_> {
    pub fn new(options) -> Self {
        CoggleApiDiagram {
            api_client: 
            id: 
            title: 
        }
    }

    pub fn replace_id(&self, url: &str) -> String {
        // ok
        url.replacen(":diagram", &self.id, 1)
    }

    pub fn web_url(&self) -> String {
        self.replace_id(&(self.api_client.base_url + "/diagram/:diagram")) // ok
    }

    pub async fn get_nodes(&self) -> Result<Vec<CoggleApiNode<'_>>, Box<dyn Error>> {
        let result: Result<Vec<CoggleNodeResource>, Box<dyn Error>> = self
            .api_client
            .get(self.replace_id("/api/1/diagrams/:diagram/nodes"), "")
            .await;

        result.map(|node_resources| {
            node_resources
                .into_iter()
                .map(|node_resource| CoggleApiNode::new(self, &node_resource))
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
    pub async fn post<T: Serialize>(&self, endpoint: &str, query_string: &str, body: &T) -> Result<Response, impl Error> {
        let prefixed_query = if query_string.starts_with('&') { 
            query_string.to_owned()
        } else {
            '&'.to_string() + query_string
        };
        HTTP_CLIENT.post(self.base_url.to_owned() + endpoint + "?access_token=" + &self.token + &prefixed_query)
            .json(&body)
            .send()
            .await
    }

    // FIXME: querystring
    pub async fn put<'de, T: Serialize + DeserializeOwned>(&self, endpoint: &str, query_string: &str, body: &T) -> Result<T, impl Error> {
        let prefixed_query = if query_string.starts_with('&') { 
            query_string.to_owned()
        } else {
            '&'.to_string() + query_string
        };
        HTTP_CLIENT.put(self.base_url.to_owned() + endpoint + "?access_token=" + &self.token + &prefixed_query)
            .json(&body)
            .send()
            .await?
            .json()
            .await
    }

    // FIXME: querystring
    pub async fn get<T: DeserializeOwned>(&self, endpoint: &str, query_string: &str) -> Result<T, impl Error> {
        let prefixed_query = if query_string.starts_with('&') { 
            query_string.to_owned()
        } else {
            '&'.to_string() + query_string
        };
        HTTP_CLIENT.get(self.base_url.to_owned() + endpoint + "?access_token=" + &self.token + &prefixed_query)
            .send()
            .await?
            .json()
            .await
    }

    // FIXME: querystring
    pub async fn delete<T: DeserializeOwned>(&self, endpoint: &str, query_string: &str) -> Result<T, impl Error> {
        let prefixed_query = if query_string.starts_with('&') { 
            query_string.to_owned()
        } else {
            '&'.to_string() + query_string
        };
        HTTP_CLIENT.get(self.base_url.to_owned() + endpoint + "?access_token=" + &self.token + &prefixed_query)
            .send()
            .await?
            .json()
            .await
    }
}

fn main() {
    println!("Hello, world!");
}
