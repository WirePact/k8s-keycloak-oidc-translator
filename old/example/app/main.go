package main

import (
	"flag"
	"fmt"
	"io/ioutil"
	"net/http"
	"os"

	"github.com/caos/oidc/pkg/client/rp"
	httphelper "github.com/caos/oidc/pkg/http"
	"github.com/caos/oidc/pkg/oidc"
	"github.com/gin-gonic/gin"
	"github.com/sirupsen/logrus"
)

type album struct {
	ID     string `json:"id"`
	Title  string `json:"title"`
	Artist string `json:"artist"`
}

var (
	port = flag.Int("port", 8000, "Port for the webserver.")
	api  = flag.String("api", "http://localhost:9000/albums", "Address for the api call.")
)

var (
	callbackPath = "/auth/callback"
	key          = []byte("test1234test1234")
)

func main() {
	flag.Parse()

	logrus.Infof("Starting webserver on port ':%v'", *port)
	logrus.Infof("Will call API on '%v'", *api)

	router := gin.Default()
	router.LoadHTMLGlob("templates/*")

	var data string
	var accessToken string

	redirectURI := fmt.Sprintf("http://localhost:%v%v", *port, callbackPath)
	cookieHandler := httphelper.NewCookieHandler(key, key, httphelper.WithUnsecure())
	options := []rp.Option{
		rp.WithCookieHandler(cookieHandler),
		rp.WithPKCE(cookieHandler),
	}

	provider, err := rp.NewRelyingPartyOIDC(os.Getenv("ISSUER"), os.Getenv("CLIENT_ID"), "", redirectURI, []string{"openid", "profile", "email"}, options...)
	if err != nil {
		panic(err)
	}

	router.GET("/", func(context *gin.Context) {
		context.HTML(http.StatusOK, "index.html", gin.H{})
	})

	router.GET("/authed", func(context *gin.Context) {
		if accessToken == "" {
			context.Redirect(302, "/login")
			return
		}

		context.HTML(http.StatusOK, "authed.html", gin.H{
			"token": accessToken,
			"data":  data,
		})

		data = ""
	})

	router.Any("/login", gin.WrapF(rp.AuthURLHandler(func() string {
		return ""
	}, provider)))

	router.Any(callbackPath, gin.WrapF(rp.CodeExchangeHandler(func(writer http.ResponseWriter, request *http.Request, tokens *oidc.Tokens, state string, relyingParty rp.RelyingParty) {
		accessToken = tokens.AccessToken
		http.Redirect(writer, request, "/authed", 302)
	}, provider)))

	router.POST("/api-call", func(context *gin.Context) {
		request, err := http.NewRequest("GET", *api, nil)
		if err != nil {
			logrus.WithError(err).Errorln("Error connecting to API.")
			data = err.Error()
			context.Redirect(http.StatusFound, "/authed")
			return
		}

		request.Header.Set("authorization", fmt.Sprintf("Bearer %v", accessToken))
		client := &http.Client{}
		response, err := client.Do(request)
		if err != nil {
			logrus.WithError(err).Errorln("Error connecting to API.")
			data = err.Error()
		} else {
			bodyText, _ := ioutil.ReadAll(response.Body)
			logrus.Infof("Fetched from API: %v", string(bodyText))
			data = string(bodyText)
		}

		context.Redirect(http.StatusFound, "/authed")
	})

	err = router.Run(fmt.Sprintf(":%v", *port))
	if err != nil {
		logrus.WithError(err).Fatal("Could not start server.")
	}
}
