# WirePact K8s Keycloak OIDC Translator

This is a "translator" for the WirePact distributed authentication mesh system.
The translator is specifically designed to work with ["Keycloak"](https://www.keycloak.org/).
It converts OIDC access tokens to a valid WirePact JWT for outgoing calls.
The translator calls the `userinfo` endpoint of the configured issuer (keycloak instance)
to fetch the subject id for the given access token. Incoming communication uses
token exchange and impersonation to fetch a valid access token for the given subject.

The keycloak instance must be started with the preview flag and the token exchange
profile to enable the features. Token exchange is in a technical preview since it is
a new part of the specification and most of the OIDC providers are still to adapt to
this change.

The configuration is done via environmental variables:

- `ISSUER`: The base url of the issuer (keycloak instance)
- `CLIENT_ID`: A valid client ID for an OIDC client inside keycloak
- `CLIENT_SECRET`: The corresponding client secret for the OIDC client
- `PKI_ADDRESS`: The address of the available WirePact PKI
- `COMMON_NAME`: The common name for the translator in the signed JWT and certificates
- `INGRESS_PORT`: Ingress communication port (default: 50051)
- `EGRESS_PORT`: Egress communication port (default: 50052)

## Keycloak configuration

To enable the system and the token exchange, the following config shall provide an example how the
system is operational:

- OIDC client for the webapp which is used to authenticate against the demo application
- OIDC client for the api
- OIDC client for the translator **with enabled service account** to enable token exchange

Special configuration:

The webapp client must be a "normal" oidc client that is configured as "public" (therefore, no client secret
exists. The authentication is done via PKCE). The translator client is an access type "confidential" client with
enabled service accounts. The service account must have the "realm-management" role "impersonation".
Finally, the api client must have a permission policy to allow the translator client to create tokens for this client.

The walk-though guide to configure keycloak for token exchange and impersonation can be found here:
https://www.keycloak.org/docs/latest/securing_apps/#internal-token-to-internal-token-exchange.

## Development

To develop the translator, use the provided docker compose file to start
all needed parts of the system. Keycloak needs around 2-3 minutes to
fully start up, so be patient ;-)

There are two users available:
- `admin` (with password `admin`): used for configuration.
- `testuser` (with password `testuser`): used to log in against the demo app.
