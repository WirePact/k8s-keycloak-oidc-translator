FROM golang:1.16-alpine as build

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

WORKDIR /app

ENV GIN_MODE=release \
    PORT=8001

EXPOSE 8001

COPY --from=build /etc/passwd /etc/group /etc/
COPY --from=build /go/bin/app /go/bin/app
COPY --from=build /app/templates /app/templates

USER appuser:appuser

ENTRYPOINT ["/go/bin/app"]
