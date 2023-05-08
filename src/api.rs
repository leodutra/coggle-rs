use regex::Regex;
use serde::{Deserialize, Serialize};

use std::{error::Error, marker::PhantomData};

use diagram::CoggleApiDiagram;
use http::HTTP_CLIENT;

use self::error::CoggleError;

pub mod diagram;
pub mod error;
pub mod http;
pub mod node;

lazy_static! {
    static ref ORG_NAME_VALIDATION: Regex =
        Regex::new(r"^[a-z]+[a-z0-9-]{2,}$").expect("Organization name validator.");
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct CoggleApi {
    pub base_url: String,
    pub token: String,
}

pub struct ApiOptions {
    pub base_url: Option<String>,
}

impl CoggleApi {
    pub fn new(options: &ApiOptions) -> Self {
        if options.token.is_empty() {
            panic!("Error: Missing api token.");
        }
        CoggleApi {
            base_url: options.base_url.unwrap_or("https://coggle.it".to_string()),
            token: options.token,
            ..Default::default()
        }
    }

    pub async fn post<T: Serialize>(
        &self,
        endpoint: &str,
        query_string: &str,
        body: &T,
    ) -> Result<CoggleApiDiagram, impl Error> {
        if !query_string.starts_with("&") {
            let query_string = format!("&{}", query_string).as_str();
        }
        HTTP_CLIENT
            .post(format!(
                "{}{}?access_token={}{}",
                self.base_url, endpoint, self.token, query_string
            ))
            .json(body)
            .send()
            .await?
            .json::<CoggleApiDiagram>()
            .await
    }

    pub async fn put<T: Serialize>(
        &self,
        endpoint: &str,
        query_string: &str,
        body: &T,
    ) -> Result<CoggleApiDiagram, impl Error> {
        if !query_string.starts_with("&") {
            let query_string = format!("&{}", query_string).as_str();
        }
        HTTP_CLIENT
            .put(format!(
                "{}{}?access_token={}{}",
                self.base_url, endpoint, self.token, query_string
            ))
            .json(body)
            .send()
            .await?
            .json::<CoggleApiDiagram>()
            .await
    }

    pub async fn get<T: Serialize>(
        &self,
        endpoint: &str,
        query_string: &str,
    ) -> Result<CoggleApiDiagram, impl Error> {
        if !query_string.starts_with("&") {
            let query_string = format!("&{}", query_string).as_str();
        }
        HTTP_CLIENT
            .get(format!(
                "{}{}?access_token={}{}",
                self.base_url, endpoint, self.token, query_string
            ))
            .send()
            .await?
            .json::<CoggleApiDiagram>()
            .await
    }

    pub async fn delete<T: Serialize>(
        &self,
        endpoint: &str,
        query_string: &str,
    ) -> Result<CoggleApiDiagram, impl Error> {
        if !query_string.starts_with("&") {
            let query_string = format!("&{}", query_string).as_str();
        }
        HTTP_CLIENT
            .delete(format!(
                "{}{}?access_token={}{}",
                self.base_url, endpoint, self.token, query_string
            ))
            .send()
            .await?
            .json::<CoggleApiDiagram>()
            .await
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

    pub async fn create_diagram(&self, title: &str) -> Result<CoggleApiDiagram, impl Error> {
        #[derive(Debug, Serialize)]
        struct Body {
            title: String,
        }

        self.post(
            "/api/1/diagrams",
            "",
            &Body {
                title: "New Diagram".to_owned(),
            },
        )
        .await
        .map(|body| CoggleApiDiagram::new(self, body))
    }
}
