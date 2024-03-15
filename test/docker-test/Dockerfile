FROM golang:1.22.1-alpine as golang

WORKDIR /app
copy . . 
RUN go mod init main.go && go mod tidy && go get . && CGO_ENABLED=0 GOOS=linux GOARCH=amd64 go build -o /server . 
FROM gcr.io/distroless/static-debian11
COPY --from=golang /server .
EXPOSE 5182
CMD ["/server"]
