package main

import (
	"context"
	"io"
	"log"
	"net/http"
	"strings"

	"github.com/seal/kappa/utils"
)

func main() {
	utils.Start(HandleRequestString)
}

/*
String input, will return "here" ( json marshals so "" is included
*/
func HandleRequestString(ctx context.Context) (string, error) {
	return "here", nil
}

/*
No struct:
Just displays the use of ctx ( getting ID and headers)
Also displays returning and IO reader ( will not be marshalled into json)
*/
func HandleRequestContext(ctx context.Context) (io.Reader, error) {
	values := ctx.Value(utils.ContextKey).(utils.ContextValues)
	log.Println("received id", values.ID)
	resp, err := http.Get("https://kimbell.uk/posts")
	if err != nil {
		return strings.NewReader("na"), err
	}
	defer resp.Body.Close()
	body, err := io.ReadAll(resp.Body)
	if err != nil {
		return strings.NewReader("here"), err
	}
	return strings.NewReader(string(body)), nil
}

/*
	Custom struct

Request must be triggered with json body that matches the struct
Response will be marshalled to json
*/
func HandleRequestEvent(ctx context.Context, event MyEvent) (Response, error) {
	log.Println("inside function, message recieved ", event.Message)

	return Response{Value: "here"}, nil
}

type Response struct {
	Value string
}
type MyEvent struct {
	Message string `json:"message"`
}
