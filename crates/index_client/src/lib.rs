use miette::{Diagnostic, Result};
use reqwest::{Client, StatusCode};
use serde::Deserialize;
use thiserror::Error;

pub struct IndexClient {
    client: Client,
    pub index: String,
}

#[derive(Error, Diagnostic, Debug)]
pub enum IndexClientError {
    #[error("Failed to send request to the index server")]
    RequestError(#[from] reqwest::Error),

    #[error("The index server returned a status code of `{0}`")]
    #[diagnostic(help("This is very likely a problem with the index server, try contacting the server administrator"))]
    StatusCodeNotOk(StatusCode),

    #[error("Failed to parse JSON returned by index server")]
    #[diagnostic(help("This is a bug. It might be a Snowflake bug, or it might be a bug with the index server, but it's a bug and should definitely be reported."))]
    JsonParsingError,

    #[error("Failed to initialize TLS backend")]
    TlsBackendInitError,

    #[error("Package not found")]
    PackageNotFound,
}

#[derive(Deserialize)]
pub struct PackageMetadata;

impl IndexClient {
    pub fn from_index_and_user_version(
        index: String,
        user_version: &str,
    ) -> Result<Self, IndexClientError> {
        let Ok(client) = Client::builder()
            .user_agent(format!(
                "SnowflakeIndexClient/{} SnowflakeCLI/{user_version}",
                env!("CARGO_PKG_VERSION")
            ))
            .build() else {
                return Err(IndexClientError::TlsBackendInitError)
            };
        Ok(Self { client, index })
    }

    pub async fn get_package(&self, name: &str) -> Result<PackageMetadata, IndexClientError> {
        let index = &self.index;
        let endpoint = format!("{index}/packages/{name}.json");
        log::debug!("Index server endpoint for package `{name}` is `{endpoint}`");

        let http_response = self
            .client
            .get(endpoint)
            .send()
            .await
            .map_err(IndexClientError::RequestError)?;

        if let Err(err) = http_response.error_for_status_ref() {
            if err.status() == Some(StatusCode::NOT_FOUND) {
                return Err(IndexClientError::PackageNotFound);
            } else {
                return Err(IndexClientError::StatusCodeNotOk(err.status().unwrap()));
            }
        }

        match http_response.json::<PackageMetadata>().await {
            Ok(package_metadata) => Ok(package_metadata),
            Err(_) => Err(IndexClientError::JsonParsingError),
        }
    }
}
