package main

import (
	"strings"

	"github.com/WirePact/go-translator"
	"github.com/WirePact/go-translator/translator"
	"github.com/WirePact/go-translator/wirepact"
	core "github.com/envoyproxy/go-control-plane/envoy/config/core/v3"
	auth "github.com/envoyproxy/go-control-plane/envoy/service/auth/v3"
	"github.com/sirupsen/logrus"
	"wirepact.ch/k8s-keycloak-oidc-translator/user_repository"
)

func main() {
	logrus.SetLevel(logrus.InfoLevel)

	err := user_repository.ConfigureOIDCRepository()
	if err != nil {
		logrus.WithError(err).Fatalln("Could not initialize OIDC repository.")
	}

	config, err := go_translator.NewConfigFromEnvironmentVariables(ingress, egress)
	if err != nil {
		logrus.WithError(err).Fatalln("Could not initialize translator config.")
	}
	server, err := go_translator.NewTranslator(&config)
	if err != nil {
		logrus.WithError(err).Fatalln("Could not create translator.")
	}

	server.Start()
}

func ingress(subject string, req *auth.CheckRequest) (translator.IngressResult, error) {
	logger := logrus.
		WithFields(logrus.Fields{
			"type":       "ingress",
			"request_id": req.Attributes.Request.Http.Id,
			"host":       req.Attributes.Request.Http.Host,
			"path":       req.Attributes.Request.Http.Path,
			"method":     req.Attributes.Request.Http.Method,
		})
	logger.Traceln("Checking ingress request.")

	accessToken, err := user_repository.GetAccessTokenForUserID(subject)
	if err != nil {
		logger.WithError(err).Errorln("Could not fetch access token for user.")
		return translator.IngressResult{}, err
	}

	return translator.IngressResult{
		HeadersToAdd: []*core.HeaderValue{
			{
				Key:   wirepact.AuthorizationHeader,
				Value: "Bearer " + accessToken,
			},
		},
	}, nil
}

func egress(req *auth.CheckRequest) (translator.EgressResult, error) {
	logger := logrus.
		WithFields(logrus.Fields{
			"type":       "egress",
			"request_id": req.Attributes.Request.Http.Id,
			"host":       req.Attributes.Request.Http.Host,
			"path":       req.Attributes.Request.Http.Path,
			"method":     req.Attributes.Request.Http.Method,
		})
	logger.Traceln("Checking egress request.")

	header, ok := req.Attributes.Request.Http.Headers[wirepact.AuthorizationHeader]
	if !ok {
		logger.Debugln("The request has no authorization header. Skipping.")
		return translator.EgressResult{Skip: true}, nil
	} else if !strings.Contains(header, "Bearer") {
		logger.Debugln("The request does not contain a Bearer token. Skipping.")
		return translator.EgressResult{Skip: true}, nil
	}

	logger.Debugln("The request contains a Bearer token. Convert to WirePact JWT.")
	accessToken := strings.ReplaceAll(header, "Bearer ", "")
	subject, err := user_repository.GetUserIDForToken(accessToken)
	if err != nil {
		logger.WithError(err).Errorln("Could not fetch userid from issuer.")
		return translator.EgressResult{}, err
	}

	return translator.EgressResult{UserID: subject, HeadersToRemove: []string{wirepact.AuthorizationHeader}}, nil
}
