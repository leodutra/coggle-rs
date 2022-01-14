
// use http::HTTP_CLIENT;
use regex::Regex;
use reqwest::Response;
use serde::{de::DeserializeOwned, Serialize};
use std::{
    error::Error,
};

use self::{http::HTTP_CLIENT, diagram::{CoggleApiDiagram, CoggleDiagramResource}, error::CoggleError};

mod diagram;
mod error;
mod http;
mod misc;
mod node;

lazy_static! {
    static ref ORG_NAME_VALIDATION: Regex =
        Regex::new(r"^[a-z]+[a-z0-9-]{2,}$").expect("Organization name validator.");
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
    pub async fn put<'de, TBody: Serialize, TResult: DeserializeOwned>(
        &self,
        endpoint: &str,
        query_string: &str,
        body: &TBody,
    ) -> Result<TResult, impl Error> {
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