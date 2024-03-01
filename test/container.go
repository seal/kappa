package main

import (
	"context"
	"fmt"
	"io"
	"log"
	"net/http"
	"strings"
	"time"

	"github.com/seal/kappa/utils"
)

func StartContainerTest() {
	var i int
	log.Println("1. Test HandleRequestString( returns string)")
	log.Println("2. Test HandleRequestContext ( returns ioReader) ")
	log.Println("3. Test handleRequestEvent ( Return json response)")
	fmt.Scanln(&i)
	switch i {
	case 1:
		go func() {
			utils.Start(HandleRequestString)
		}()
		time.Sleep(2 * time.Second)
		handleRequestString()
	case 2:
		go func() {
			utils.Start(HandleRequestContext)
		}()
		time.Sleep(2 * time.Second)
		handleRequestContext()
	case 3:
		go func() {
			utils.Start(HandleRequestEvent)
		}()
		time.Sleep(2 * time.Second)
		handleRequestEvent()
	}
}
func handleRequestEvent() {
	req, err := http.NewRequest("POST", containerURL, strings.NewReader(`{ "message":"message here"}`))
	if err != nil {
		panic(err)
	}
	req.Header.Set("Content-Type", "application/json")
	values := req.URL.Query()
	values.Add("id", "id here")
	req.URL.RawQuery = values.Encode()
	client := &http.Client{}
	resp, err := client.Do(req)
	if err != nil {
		panic(err)
	}
	defer resp.Body.Close()
	body, err := io.ReadAll(resp.Body)
	if err != nil {
		panic(err)
	}
	if resp.StatusCode != 200 {
		panic(fmt.Errorf("status code not 200 , status code: %d\n Body:%s", resp.StatusCode, string(body)))
	}
	log.Println(string(body))
}
func handleRequestContext() {
	req, err := http.NewRequest("POST", containerURL, nil)
	if err != nil {
		panic(err)
	}
	req.Header.Set("Content-Type", "application/json")
	values := req.URL.Query()
	values.Add("id", "id here")
	req.URL.RawQuery = values.Encode()
	client := &http.Client{}
	resp, err := client.Do(req)
	if err != nil {
		panic(err)
	}
	defer resp.Body.Close()
	body, err := io.ReadAll(resp.Body)
	if err != nil {
		panic(err)
	}
	if resp.StatusCode != 200 {
		panic(fmt.Errorf("status code not 200 , status code: %d\n Body:%s", resp.StatusCode, string(body)))
	}
	log.Println(string(body))
}
func handleRequestString() {
	req, err := http.NewRequest("POST", containerURL, nil)
	if err != nil {
		panic(err)
	}
	req.Header.Set("Content-Type", "application/json")
	values := req.URL.Query()
	values.Add("id", "id here")
	req.URL.RawQuery = values.Encode()
	client := &http.Client{}
	resp, err := client.Do(req)
	if err != nil {
		panic(err)
	}
	defer resp.Body.Close()
	body, err := io.ReadAll(resp.Body)
	if err != nil {
		panic(err)
	}
	if resp.StatusCode != 200 {
		panic(fmt.Errorf("status code not 200 , status code: %d\n Body:%s", resp.StatusCode, string(body)))
	}
	log.Println(string(body))
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

	return Response{Value: "Returning message received - " + event.Message}, nil
}

type Response struct {
	Value string
}
type MyEvent struct {
	Message string `json:"message"`
}
