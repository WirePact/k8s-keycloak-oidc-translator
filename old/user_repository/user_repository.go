package user_repository

import (
	"errors"
	"fmt"
	"net/http"
	"net/url"
	"os"
	"strings"

	"github.com/caos/oidc/pkg/client"
	httphelper "github.com/caos/oidc/pkg/http"
	"github.com/caos/oidc/pkg/oidc"
)

var (
	endpoints    *oidc.DiscoveryConfiguration
	issuer       = os.Getenv("ISSUER")
	clientId     = os.Getenv("CLIENT_ID")
	clientSecret = os.Getenv("CLIENT_SECRET")
)

func ValidateOIDCRepository() error {
	if issuer == "" {
		return errors.New("issuer not set")
	}

	if clientId == "" {
		return errors.New("clientId not set")
	}

	if clientSecret == "" {
		return errors.New("clientSecret not set")
	}

	return nil
}

func ConfigureOIDCRepository() error {
	var err error

	endpoints, err = client.Discover(issuer, &http.Client{})
	if err != nil {
		return err
	}

	err = ValidateOIDCRepository()
	if err != nil {
		return err
	}

	return nil
}

func GetUserIDForToken(accessToken string) (string, error) {
	request, err := http.NewRequest("GET", endpoints.UserinfoEndpoint, nil)
	if err != nil {
		return "", err
	}

	request.Header.Set("authorization", fmt.Sprintf("Bearer %v", accessToken))
	userinfo := oidc.NewUserInfo()
	if err := httphelper.HttpRequest(&http.Client{}, request, &userinfo); err != nil {
		return "", err
	}

	return userinfo.GetSubject(), nil
}

func getServiceAccountAccessToken() (string, error) {
	values := url.Values{}
	values.Set("grant_type", "client_credentials")
	request, err := http.NewRequest("POST", endpoints.TokenEndpoint, strings.NewReader(values.Encode()))
	if err != nil {
		return "", err
	}
	request.Header.Add("Content-Type", "application/x-www-form-urlencoded")
	request.SetBasicAuth(clientId, clientSecret)

	response := oidc.AccessTokenResponse{}
	if err := httphelper.HttpRequest(&http.Client{}, request, &response); err != nil {
		return "", err
	}

	return response.AccessToken, nil
}

func GetAccessTokenForUserID(userID string) (string, error) {
	serviceAccountToken, err := getServiceAccountAccessToken()
	if err != nil {
		return "", err
	}

	values := url.Values{}
	values.Set("grant_type", "urn:ietf:params:oauth:grant-type:token-exchange")
	values.Set("subject_token", serviceAccountToken)
	values.Set("subject_token_type", "urn:ietf:params:oauth:token-type:access_token")
	values.Set("requested_subject", userID)
	values.Set("requested_token_type", "urn:ietf:params:oauth:token-type:access_token")
	request, err := http.NewRequest("POST", endpoints.TokenEndpoint, strings.NewReader(values.Encode()))
	if err != nil {
		return "", err
	}
	request.Header.Add("Content-Type", "application/x-www-form-urlencoded")
	request.SetBasicAuth(clientId, clientSecret)

	response := oidc.AccessTokenResponse{}
	if err := httphelper.HttpRequest(&http.Client{}, request, &response); err != nil {
		return "", err
	}

	return response.AccessToken, nil
}
