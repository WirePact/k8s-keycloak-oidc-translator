use std::fmt::format;
use std::sync::Arc;

use clap::{ArgEnum, Parser};
use log::{debug, error, info};
use tokio::sync::Mutex;
use wirepact_translator::{
    run_translator, CheckRequest, EgressResult, IngressResult, Status, Translator,
    TranslatorConfig, HTTP_AUTHORIZATION_HEADER,
};

use crate::provider::{ClientCredentialProvider, Provider};

mod provider;

#[derive(Clone, Debug, ArgEnum)]
enum AuthType {
    ClientCredentials,
    JWTProfile,
}

#[derive(Parser, Debug)]
#[clap(version, about, long_about = None)]
struct Cli {
    /// The address of the WirePact PKI.
    #[clap(short, long, env)]
    pki_address: String,

    /// The name of the translator. This is used as common name
    /// when requesting a certificate from the PKI.
    #[clap(short, long, env, default_value = "k8s oidc token exchange translator")]
    name: String,

    /// The port that the server will listen for
    /// ingress communication (incoming connections) on.
    #[clap(short, long, env, default_value = "50051")]
    ingress_port: u16,

    /// The port that the server will listen for
    /// egress communication (outgoing connections) on.
    #[clap(short, long, env, default_value = "50052")]
    egress_port: u16,

    /// If set, debug log messages are printed as well.
    #[clap(short, long, env)]
    debug: bool,

    /// The issuer for OIDC tokens. This is used in conjunction
    /// with the token-exchange grant to fetch an access token
    /// on the users behalf. If `discovery_url` is not set, this
    /// url will be used with `/.well-known/openid-configuration`
    /// to fetch the endpoint urls.
    #[clap(long, env)]
    issuer: String,

    /// Optional overwrite of the well-known discovery document endpoint
    /// of the issuer. If set, please provide the full url to the OIDC
    /// discovery document.
    #[clap(long, env)]
    discovery_url: Option<String>,

    /// Determine the authentication type for the issuer. There exist
    /// two authentication types:
    ///
    /// - Client Credentials: The translator will use client ID and
    ///   client secret to authenticate itself against the issuer.
    ///
    /// - JWT Profile: The translator will use a JWT profile
    ///   ([RFC7523](https://datatracker.ietf.org/doc/html/rfc7523)) to authenticate
    ///   itself against the issuer.
    ///
    /// Depending on the selected auth type, other parameters are required.
    #[clap(arg_enum, long, env)]
    auth_type: AuthType,

    /// Required if auth_type is set to [AuthType::ClientCredentials].
    /// Defines the client ID to use when authenticating against the issuer.
    #[clap(long, env)]
    client_id: Option<String>,

    /// Required if auth_type is set to [AuthType::ClientCredentials].
    /// Defines the client secret to use when authenticating against the issuer.
    #[clap(long, env)]
    client_secret: Option<String>,

    /// Required if auth_type is set to [AuthType::JWTProfile].
    /// Defines the file path to the JWT profile to use when authenticating against the issuer.
    /// The profile must be a JSON file containing the following fields:
    ///
    /// - userId: The user ID of the machine account.
    ///
    /// - keyId: The ID of the used signing RSA key.
    ///
    /// - key: The pem encoded RSA (private) key.
    #[clap(long, env)]
    jwt_profile_path: Option<String>,
    // TODO: token endpoint auth type must be specified (client secret basic auth, client secret post, private key jwt)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    let level = match cli.debug {
        true => log::LevelFilter::Debug,
        false => log::LevelFilter::Info,
    };

    env_logger::builder()
        .filter_module("k8s_token_exchange_translator", level)
        .filter_module("wirepact_translator", level)
        .init();

    info!("Starting oidc token exchange translator '{}'.", cli.name);
    debug!("Debug logging is enabled.");

    let url = format!("{}/.well-known/openid-configuration", cli.issuer);

    run_translator(&TranslatorConfig {
        pki_address: cli.pki_address,
        common_name: cli.name,
        ingress_port: cli.ingress_port,
        egress_port: cli.egress_port,
        translator: Arc::new(OidcTranslator {
            authenticator: Arc::new(Mutex::new(
                ClientCredentialProvider::new(
                    &url,
                    cli.client_id.unwrap(),
                    cli.client_secret.unwrap(),
                )
                .await?,
            )),
        }),
    })
    .await?;

    Ok(())
}

struct OidcTranslator {
    authenticator: Arc<Mutex<dyn Provider>>,
}

#[wirepact_translator::async_trait]
impl Translator for OidcTranslator {
    async fn ingress(&self, subject_id: &str, _: &CheckRequest) -> Result<IngressResult, Status> {
        let access_token = self
            .authenticator
            .lock()
            .await
            .access_token_for_user_id(subject_id)
            .await
            .map_err(|e| {
                error!(
                    "Failed to get access token for user ID '{}': {}",
                    subject_id, e
                );
                Status::internal(format!(
                    "Failed to get access token for user ID '{}': {}",
                    subject_id, e
                ))
            })?;

        debug!("Fetched access token for user with ID '{}'.", subject_id);

        Ok(IngressResult::allowed(
            Some(vec![(
                HTTP_AUTHORIZATION_HEADER.to_string(),
                format!("Bearer {}", access_token),
            )]),
            None,
        ))
    }

    async fn egress(&self, request: &CheckRequest) -> Result<EgressResult, Status> {
        let auth_header = self.get_header(request, HTTP_AUTHORIZATION_HEADER)?;

        if auth_header.is_none() {
            debug!("No authorization header found. Skip request.");
            return Ok(EgressResult::skip());
        }

        let auth_header = auth_header.unwrap();
        if !auth_header.starts_with("Bearer ") {
            debug!("Authorization header does not start with 'Bearer'. Skip Request.");
            return Ok(EgressResult::skip());
        }

        let access_token = auth_header.replace("Bearer ", "");
        let subject = self
            .authenticator
            .lock()
            .await
            .user_id_for_token(&access_token)
            .await
            .map_err(|e| {
                error!("Failed to get user ID for access token: {}", e);
                Status::internal(format!("Failed to get user ID for access token: {}", e))
            })?;

        debug!("Fetched user ID '{}' from access token.", subject);

        Ok(EgressResult::allowed(
            subject,
            Some(vec![HTTP_AUTHORIZATION_HEADER.to_string()]),
        ))
    }
}
