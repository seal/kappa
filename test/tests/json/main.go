package main

import (
	"context"
	"github.com/seal/kappa/utils"
)

func main() {
	utils.Start(HandleRequestEvent)
}

/*
	Custom struct

Request must be triggered with json body that matches the struct
Response will be marshalled to json
*/
func HandleRequestEvent(ctx context.Context, event MyEvent) (Response, error) {

	return Response{Value: "Returning message received - " + event.Message}, nil
}

type Response struct {
	Value string
}
type MyEvent struct {
	Message string `json:"message"`
}
