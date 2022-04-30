# WirePact K8s Token Exchange Translator

This is a "translator" for the WirePact distributed authentication mesh system.
The translator uses OAuth2.0 Token Exchange ([RFC8693](https://tools.ietf.org/html/rfc8693))
to convert OAuth (or OIDC) tokens to the WirePact common language format (signed JWT)
and back.

For egress communication, the user-info endpoint of the configured issuer is used
to fetch the user ID with the provided token. For ingress communication, the provided
client credentials are used to fetch a machine access token and then exchange the
token via Token Exchange to obtain an access token for the subject.

## Token Exchange

OAuth2.0 Token Exchange, as defined in [RFC8693](https://tools.ietf.org/html/rfc8693), allows
a machine user to obtain a valid access token for any given user (provided the machine user
has the necessary permissions). The token exchange is performed by the following rules:

1. ``HTTP_POST`` call to the ``Token Endpoint`` of the issuer.
2. Depending on the authorization method of the configured oidc client, provide ``client id``
   and ``client secret`` or a ``jwt profile``.
3. Set the following url encoded form values:
    - ``grant_type``: ``urn:ietf:params:oauth:grant-type:token-exchange``
    - ``subject_token_type``: ``urn:ietf:params:oauth:token-type:access_token``
    - ``subject_token``: the access token of the machine user
    - ``request_token_type``: ``urn:ietf:params:oauth:token-type:access_token``
    - ``requested_subject``: subject of the target user

## Configuration

The configuration is done via environmental variables or command line arguments:

- ``PKI_ADDRESS`` (``-p --pki-address <PKI_ADDRESS>``): The address of the available WirePact PKI.
- ``NAME`` (``-n --name <EGRESS_PORT>``): The common name for the translator that is used for certificates and
  signing JWT tokens (default: ``k8s basic auth translator``).
- ``INGRESS_PORT`` (``-i --ingress-port <INGRESS_PORT>``): Ingress communication port (default: 50051).
- ``EGRESS_PORT`` (``-e --egress-port <EGRESS_PORT>``): Egress communication port (default: 50052).
- ``DEBUG`` (``-d --debug``): Enable debug logging.
- ``ISSUER`` (``--issuer <ISSUER>``): The issuer address for OAuth/OIDC operations.
- ``DISCOVERY_URL`` (``--discovery-url <DISCOVERY_URL>``): The discovery url for the issuer. If omitted, the
  well-known issuer discovery url (``<ISSUER>/.well-known/openid-configuration``) is used.
- ``AUTH_TYPE`` (``--auth-type <AUTH_TYPE>``): The authorization type for the configured oidc client.
  Supported values are ``client-credentials`` and ``jwt-profile``. Currently, only ``client-credentials`` is
  implemented.
- ``CLIENT_ID`` (``--client-id <CLIENT_ID>``): The client id for the configured client. This value is required if
  ``AUTH_TYPE`` is set to ``client-credentials``.
- ``CLIENT_SECRET`` (``--client-secret <CLIENT_SECRET>``): The client secret for the configured client. This value is
  required if ``AUTH_TYPE`` is set to ``client-credentials``.
