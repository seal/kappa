package main

import (
	"context"
	"log"

	"github.com/seal/kappa/utils"
)

func main() {
	utils.Start(HandleRequest)
}

func HandleRequest(ctx context.Context, event interface{}) (string, error) {
	field, ok := event.(MyEvent)
	if !ok {
		log.Println("Not ok")
	}
	log.Println(field.Message)
	return "ahh", nil
}

type MyEvent struct {
	Message string `json:"message"`
}

/*
package main

import (
	"context"
	"fmt"
	"github.com/aws/aws-lambda-go/lambda"
)

type MyEvent struct {
	Name string `json:"name"`
}

func HandleRequest(ctx context.Context, event *MyEvent) (*string, error) {
	if event == nil {
		return nil, fmt.Errorf("received nil event")
	}
	message := fmt.Sprintf("Hello %s!", event.Name)
	return &message, nil
}

func main() {
	lambda.Start(HandleRequest)
}
*/
