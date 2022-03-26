FROM golang:1.18-alpine as build

WORKDIR /app

ENV GOOS=linux \
    GOARCH=amd64 \
    USER=appuser \
    UID=1000

RUN adduser \
    --disabled-password \
    --gecos "" \
    --home "/nonexistent" \
    --shell "/sbin/nologin" \
    --no-create-home \
    --uid "${UID}" \
    "${USER}"

COPY go.mod go.sum ./
RUN go mod download && go mod verify

COPY . .

RUN go build -ldflags="-w -s" -o /go/bin/app


FROM alpine

ARG BUILD_VERSION
ARG COMMIT_SHA

LABEL org.opencontainers.image.source="https://github.com/WirePact/k8s-keycloak-oidc-translator" \
    org.opencontainers.image.authors="cbuehler@rootd.ch" \
    org.opencontainers.image.url="https://github.com/WirePact/k8s-keycloak-oidc-translator" \
    org.opencontainers.image.documentation="https://github.com/WirePact/k8s-keycloak-oidc-translator/blob/main/README.md" \
    org.opencontainers.image.source="https://github.com/WirePact/k8s-keycloak-oidc-translator/blob/main/Dockerfile" \
    org.opencontainers.image.version="${BUILD_VERSION}" \
    org.opencontainers.image.revision="${COMMIT_SHA}" \
    org.opencontainers.image.licenses="Apache-2.0" \
    org.opencontainers.image.title="WirePact Kubernetes Keycloak OIDC Translator" \
    org.opencontainers.image.description="Translator for WirePact that handles OIDC authentication for a Keycloak instance for any software behind."

WORKDIR /app

ENV BUILD_VERSION=${BUILD_VERSION}

COPY --from=build /etc/passwd /etc/group /etc/
COPY --from=build /go/bin/app /app/app

RUN chown -R appuser:appuser /app

USER appuser:appuser

ENTRYPOINT ["/app/app"]
