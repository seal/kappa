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
	return Response{MessageOne: event.Message, MessageTwo: event.MessageTwo}, nil
}

type Response struct {
	MessageOne string `json:"messageOne"`
	MessageTwo string `json:"messagewTwo"`
}
type MyEvent struct {
	Message    string `json:"message"`
	MessageTwo string `json:"messageTwo"`
}

