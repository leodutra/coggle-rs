use serde::{Serialize, Deserialize};

use super::{diagram::CoggleApiDiagram, http::HTTP_CLIENT};
use std::{error::Error};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CoggleApi {
    pub base_url: String,
    pub token: String,
}


pub struct ApiOptions {
    pub base_url: Option<String>,
    pub token: String,
}

impl CoggleApi {
    pub fn new(options: &ApiOptions) -> Self {
        CoggleApi {
            base_url: options.base_url.unwrap_or("https://coggle.it".to_string()),
            token: options.token,
        }
    }

    pub async fn post(
        &self,
        endpoint: &str,
        query_string: &str,
        body: &str,
    ) -> Result<CoggleApiDiagram, impl Error> {
        if !query_string.starts_with("&") {
            let query_string = format!("&{}", query_string).as_str();
        }
        HTTP_CLIENT
            .post(format!(
                "{}{}?access_token={}{}",
                self.base_url, endpoint, self.token, query_string
            ))
            // .query(&[("access_token", token)])
            .send()
            .await?
            .json::<CoggleApiDiagram>()
            .await
    }

    pub async fn put(
        &self,
        endpoint: &str,
        query_string: &str,
        body: &str,
    ) -> Result<CoggleApiDiagram, impl Error> {
        if !query_string.starts_with("&") {
            let query_string = format!("&{}", query_string).as_str();
        }
        HTTP_CLIENT
            .put(format!(
                "{}{}?access_token={}{}",
                self.base_url, endpoint, self.token, query_string
            ))
            // .query(&[("access_token", token)])
            .send()
            .await?
            .json::<CoggleApiDiagram>()
            .await
    }

    pub async fn create_diagram(&self, title: &str) -> Result<CoggleApiDiagram, impl Error> {
        self.post(
            "/api/1/diagrams",
            "",
            json!({
                title: "New Diagram",
            }),
        )
        .await?
        .map(|body| CoggleApiDiagram::new(self, body))
    }
}
