use std::sync::Arc;

use clap::{ArgEnum, Parser};
use log::{debug, error, info, warn};
use wirepact_translator::{
    run_translator, CheckRequest, EgressResult, IngressResult, Status, Translator, TranslatorConfig,
};

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
    /// on the users behalf.
    #[clap(long, env)]
    issuer: String,

    /// Optional overwrite of the well-known discovery document endpoint
    /// of the [issuer].
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

    /// Required if [auth_type] is set to [AuthType::ClientCredentials].
    /// Defines the client ID to use when authenticating against the issuer.
    #[clap(long, env)]
    client_id: Option<String>,

    /// Required if [auth_type] is set to [AuthType::ClientCredentials].
    /// Defines the client secret to use when authenticating against the issuer.
    #[clap(long, env)]
    client_secret: Option<String>,

    /// Required if [auth_type] is set to [AuthType::JWTProfile].
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
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    let level = match cli.debug {
        true => log::LevelFilter::Debug,
        false => log::LevelFilter::Info,
    };

    env_logger::builder()
        .filter_module("k8s_oidc_token_exchange_translator", level)
        .filter_module("wirepact_translator", level)
        .init();

    info!("Starting oidc token exchange translator '{}'.", cli.name);
    debug!("Debug logging is enabled.");

    // let mut repository: Option<Arc<dyn Repository>> = None;
    //
    // if let Mode::Csv = cli.mode {
    //     if cli.csv_path.is_none() {
    //         error!("No CSV path provided.");
    //         return Err("No CSV path provided.".into());
    //     }
    //
    //     repository = Some(Arc::new(LocalCsvRepository::new(&cli.csv_path.unwrap())?));
    // }
    //
    // if let Mode::Kubernetes = cli.mode {
    //     if cli.k8s_secret_name.is_none() {
    //         error!("No Kubernetes secret name provided.");
    //         return Err("No Kubernetes secret name provided.".into());
    //     }
    //
    //     repository = Some(Arc::new(
    //         KubernetesSecretRepository::new(&cli.k8s_secret_name.unwrap()).await?,
    //     ));
    // }

    run_translator(&TranslatorConfig {
        pki_address: cli.pki_address,
        common_name: cli.name,
        ingress_port: cli.ingress_port,
        egress_port: cli.egress_port,
        translator: Arc::new(OidcTranslator {}),
    })
    .await?;

    Ok(())
}

struct OidcTranslator {}

#[wirepact_translator::async_trait]
impl Translator for OidcTranslator {
    async fn ingress(&self, subject_id: &str, _: &CheckRequest) -> Result<IngressResult, Status> {
        Ok(IngressResult::skip())
    }

    async fn egress(&self, request: &CheckRequest) -> Result<EgressResult, Status> {
        Ok(EgressResult::skip())
    }
}
