package main

import (
	"errors"
	"flag"
	"fmt"
	"os"
	"strings"

	"github.com/caos/oidc/pkg/client/rs"
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
	port = flag.Int("port", 9000, "Port for the webserver.")
)

var data = [...]album{
	{ID: "one", Title: "39 Seconds", Artist: "Marcus Warner"},
	{ID: "two", Title: "Born To Lose", Artist: "TLMT"},
	{ID: "three", Title: "Midgard (Tour Edition)", Artist: "Faun"},
}

func main() {
	flag.Parse()

	logrus.Infof("Starting webserver on port ':%v'", *port)

	router := gin.Default()
	resourceServer, err := rs.NewResourceServerClientCredentials(os.Getenv("ISSUER"), os.Getenv("CLIENT_ID"), os.Getenv("CLIENT_SECRET"))
	if err != nil {
		panic(err)
	}

	router.GET("albums", func(context *gin.Context) {
		header := context.Request.Header.Get("authorization")
		if header == "" {
			context.AbortWithError(401, errors.New("unauthorized"))
			return
		}

		if !strings.HasPrefix(header, oidc.PrefixBearer) {
			context.AbortWithError(401, errors.New("unauthorized"))
			return
		}

		token := strings.TrimPrefix(header, oidc.PrefixBearer)

		resourceServer.IntrospectionURL()
		_, err := rs.Introspect(context.Request.Context(), resourceServer, token)
		if err != nil {
			context.AbortWithError(403, err)
			return
		}

		context.JSON(200, data)
	})

	err = router.Run(fmt.Sprintf(":%v", *port))
	if err != nil {
		logrus.WithError(err).Fatal("Could not start server.")
	}
}
