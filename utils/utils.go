package utils

import (
	"bytes"
	"context"
	"errors"
	"io"
	"log"
	"reflect"
	"strings"
)

func Start(handler interface{}) {
	/*j := []byte(` { "message":"message"}`)
	var x interface{}
	err := json.Unmarshal(j, &x)
	if err != nil {
		panic(err)
	}
	handler(context.Background(), x)
	*/
	h := Parsehandler(handler)
	x, err := h(context.Background(), []byte("asd"))
	if err != nil {
		panic(err)
	}
	log.Println("got ", x, " from func")
}

//type HandlerFunc func(context.Context, interface{}) (string, error)

type jsonOutBuffer struct {
	*bytes.Buffer
}

func Parsehandler(f interface{}) handlerFunc {
	handler := reflect.ValueOf(f)
	handlerType := reflect.TypeOf(f)
	log.Println("Got Type", handlerType)
	log.Println("Got handler", handler)
	if handlerType.Kind() != reflect.Func {
		return errorHandler(errors.New("Not a function"))
	}

	//out := &jsonOutBuffer{bytes.NewBuffer(nil)}
	return func(ctx context.Context, payload []byte) (io.Reader, error) {

		//payload := []byte("example payload")

		ctxValue := reflect.ValueOf(ctx)
		payloadValue := reflect.ValueOf(payload)

		response := handler.Call([]reflect.Value{ctxValue, payloadValue})
		if len(response) > 0 {
			if errVal, ok := response[len(response)-1].Interface().(error); ok && errVal != nil {
				return nil, errVal
			}
		}
		return strings.NewReader(""), nil
	}
}

type handlerFunc func(context.Context, []byte) (io.Reader, error)

func errorHandler(err error) handlerFunc {
	return func(_ context.Context, _ []byte) (io.Reader, error) {
		return nil, err
	}
} /*
func Start(f func(t ...StrInput) (string, error)) {
	return1, err := f()
	if err != nil {
		log.Println("error running handler, send logs to rust daemon here")
	}
	log.Println(return1)
}

type StrInput struct {
	key   string
	value string
}
*/

/*

 */
/*
func Start(f func() (string, error)) {
	return1, err := f()
	if err != nil {
		log.Println("error running handler, send logs to rust daemon here")
	}
	log.Println(return1)
}
*/

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
