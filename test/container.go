package main

import (
	"fmt"
	"io"
	"log"
	"net/http"
	"strings"
)

func StartContainerTest() {
	var i int
	log.Println("1. Test HandleRequestString( returns string)")
	log.Println("2. Test HandleRequestContext ( returns ioReader) ")
	log.Println("3. Test handleRequestEvent ( Return json response)")
	fmt.Scanln(&i)
	switch i {
	case 1:
		handleRequestString()
	case 2:
		handleRequestContext()
	case 3:
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
