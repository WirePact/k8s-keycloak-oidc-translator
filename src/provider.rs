use std::error::Error;
use std::ops::Add;
use std::time::{Duration, SystemTime};

use log::debug;
use reqwest::Client;
use serde::Deserialize;
use wirepact_translator::HTTP_AUTHORIZATION_HEADER;

#[wirepact_translator::async_trait]
pub(crate) trait Provider: Send + Sync {
    async fn user_id_for_token(&self, token: &str) -> Result<String, Box<dyn Error>> {
        let client = self.client();
        let user_info = client
            .get(&self.discovery().userinfo_endpoint)
            .header(HTTP_AUTHORIZATION_HEADER, format!("Bearer {}", token))
            .send()
            .await?
            .json::<UserInfo>()
            .await?;

        Ok(user_info.sub)
    }

    async fn access_token_for_user_id(&mut self, user_id: &str) -> Result<String, Box<dyn Error>>;

    fn client(&self) -> &reqwest::Client;

    fn discovery(&self) -> &DiscoveryDocument;
}

pub(crate) struct ClientCredentialProvider {
    http: reqwest::Client,
    client_id: String,
    client_secret: String,
    discovery: DiscoveryDocument,
    access_token: Option<String>,
    access_token_expiration: Option<SystemTime>,
}

impl ClientCredentialProvider {
    pub(crate) async fn new(
        discovery_url: &str,
        client_id: String,
        client_secret: String,
    ) -> Result<Self, Box<dyn Error>> {
        let client = reqwest::Client::new();
        let discovery = client
            .get(discovery_url)
            .send()
            .await?
            .json::<DiscoveryDocument>()
            .await?;

        Ok(Self {
            http: client,
            client_id,
            client_secret,
            discovery,
            access_token: None,
            access_token_expiration: None,
        })
    }

    async fn get_access_token(&mut self) -> Result<String, Box<dyn Error>> {
        if let (Some(token), Some(expiration)) = (&self.access_token, &self.access_token_expiration)
        {
            if &SystemTime::now().add(Duration::from_secs(10)) < expiration {
                debug!("Access token for machine account still valid.");
                return Ok(token.clone());
            }
        }

        let response = self
            .client()
            .post(&self.discovery.token_endpoint)
            .basic_auth(&self.client_id, Some(&self.client_secret))
            .form(&[("grant_type", "client_credentials")])
            .send()
            .await?
            .json::<AccessTokenResponse>()
            .await?;

        let expiration = SystemTime::now().add(Duration::from_secs(response.expires_in));
        self.access_token = Some(response.access_token.clone());
        self.access_token_expiration = Some(expiration);

        debug!("Cache machine access token for {}s.", response.expires_in);

        Ok(response.access_token)
    }
}

#[wirepact_translator::async_trait]
impl Provider for ClientCredentialProvider {
    async fn access_token_for_user_id(&mut self, user_id: &str) -> Result<String, Box<dyn Error>> {
        let access_token = self.get_access_token().await?;
        let response = self
            .client()
            .post(&self.discovery.token_endpoint)
            .basic_auth(&self.client_id, Some(&self.client_secret))
            .form(&[
                (
                    "grant_type",
                    "urn:ietf:params:oauth:grant-type:token-exchange",
                ),
                (
                    "subject_token_type",
                    "urn:ietf:params:oauth:token-type:access_token",
                ),
                ("subject_token", &access_token),
                ("requested_subject", user_id),
                (
                    "requested_token_type",
                    "urn:ietf:params:oauth:token-type:access_token",
                ),
            ])
            .send()
            .await?
            .json::<AccessTokenResponse>()
            .await?;

        Ok(response.access_token)
    }

    fn client(&self) -> &Client {
        &self.http
    }

    fn discovery(&self) -> &DiscoveryDocument {
        &self.discovery
    }
}

// struct JWTProfileProvider;
//
// impl JWTProfileProvider {}
//
// #[wirepact_translator::async_trait]
// impl Provider for JWTProfileProvider {}

#[derive(Deserialize)]
pub(crate) struct DiscoveryDocument {
    token_endpoint: String,
    userinfo_endpoint: String,
}

#[derive(Deserialize)]
struct UserInfo {
    sub: String,
}

#[derive(Deserialize)]
struct AccessTokenResponse {
    access_token: String,
    expires_in: u64,
}
