package main

import (
	"bytes"
	"encoding/json"
	"fmt"
	"io"
	"log"
	"mime/multipart"
	"net/http"
	"net/textproto"
	"os"

	"github.com/google/uuid"
)

func StartEpTest() {
	for {
		var input int
		log.Println("Enter an option")
		log.Println("1. Register and save api-key to env")
		log.Println("2. Get user ")
		log.Println("3. Get containers")
		log.Println("4. Create container")
		log.Println("5. Trigger container")
		fmt.Scanln(&input)
		switch input {
		case 1:
			register()
		case 2:
			getUser()
		case 3:
			getContainers()
		case 4:
			var i2 int
			log.Println("Enter an option")
			log.Println("1: dummy data")
			log.Println("2: input data")
			fmt.Scanln(&i2)
			switch i2 {
			case 1:
				createContainer("go", 1234, "test_zip.zip")
			case 2:
				var l string
				log.Println("enter language")
				fmt.Scanln(&l)
				var f string
				log.Println("enter filename")
				fmt.Scanln(&f)
				var port int
				log.Println("enter port")
				fmt.Scanln(&port)
				createContainer(l, port, f)
			}
		case 5:
			c := getContainers()
			triggerContainer(c)
		case 0:
			os.Exit(1)
		}
	}
}
func triggerContainer(c []container) {
	log.Println("select a container to trigger")
	for k := range c {
		log.Printf("%d - %s\n", k+1, c[k].ContainerID)
	}
	var cIndex int
	fmt.Scanln(&cIndex)
	// post /trigger query param container_id
	req, err := http.NewRequest("POST", fmt.Sprintf("%s/trigger", baseURL), nil)
	if err != nil {
		panic(err)
	}
	apiKey := getAPIKey()
	req.Header.Set("Content-Type", "application/json")
	req.Header.Set("API-KEY", apiKey)
	values := req.URL.Query()
	values.Add("container_id", c[cIndex].ContainerID.String())
	req.URL.RawQuery = values.Encode()
	client := &http.Client{}
	resp, err := client.Do(req)
	if err != nil {
		panic(err)
	}
	if resp.StatusCode != 200 {
		panic(fmt.Errorf("status code not 200 , status code: %d", resp.StatusCode))
	}
	defer resp.Body.Close()
	body, err := io.ReadAll(resp.Body)
	if err != nil {
		panic(err)
	}
	log.Println(string(body))
}

func createContainer(language string, port int, filePath string) {
	apiKey := getAPIKey()
	log.Println("Got api key", apiKey)

	body := &bytes.Buffer{}
	writer := multipart.NewWriter(body)

	header := make(textproto.MIMEHeader)

	header.Set("API-KEY", apiKey)

	file, err := os.Open(filePath)
	if err != nil {
		panic(err)
	}
	defer file.Close()

	part, err := writer.CreateFormFile("file", filePath)
	if err != nil {
		panic(err)
	}
	_, err = io.Copy(part, file)
	if err != nil {
		panic(err)
	}

	writer.Close()

	req, err := http.NewRequest("POST", fmt.Sprintf("%s/containers?language=%s&port=%d", baseURL, language, port), body)
	if err != nil {
		panic(err)
	}
	req.Header.Set("Content-Type", writer.FormDataContentType())
	req.Header.Set("API-KEY", apiKey)

	client := &http.Client{}
	resp, err := client.Do(req)
	if err != nil {
		panic(err)
	}
	defer resp.Body.Close()

	bodyResp, err := io.ReadAll(resp.Body)
	if err != nil {
		panic(err)
	}
	log.Println("Response:", string(bodyResp))

	if resp.StatusCode != 200 {
		panic(fmt.Errorf("status code not 200 %d", resp.StatusCode))
	}
}
func getContainers() []container {
	apiKey := getAPIKey()
	log.Println("Got api key", apiKey)

	req, err := http.NewRequest("GET", fmt.Sprintf("%s/containers", baseURL), nil)
	if err != nil {
		panic(err)
	}
	req.Header.Set("Content-Type", "application/json")
	req.Header.Set("API-KEY", apiKey)

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
	log.Println("Body", string(body))

	var containers []container
	err = json.Unmarshal(body, &containers)
	if err != nil {
		panic(err)
	}

	log.Println("Got containers:")
	for _, c := range containers {
		log.Printf("Container ID: %s, Language: %s, Port: %d\n", c.ContainerID, c.Language, c.Port)
	}
	return containers
}

// Container struct definition
type container struct {
	ContainerID uuid.UUID `json:"container_id"`
	Language    string    `json:"language"`
	Port        int       `json:"port"`
}

func getAPIKey() string {
	f, err := os.ReadFile("env.json")
	if err != nil {
		panic(err)
	}
	var e env
	err = json.Unmarshal(f, &e)
	if err != nil {
		panic(err)
	}
	return e.ApiKey
}
func getUser() {
	apiKey := getAPIKey()
	log.Println("Got api key", apiKey)
	req, err := http.NewRequest("GET", fmt.Sprintf("%s/user", baseURL), nil)
	if err != nil {
		panic(err)
	}
	req.Header.Set("Content-Type", "application/json")
	req.Header.Set("API-KEY", apiKey)
	client := &http.Client{}
	resp, err := client.Do(req)
	if err != nil {
		panic(err)
	}
	body, err := io.ReadAll(resp.Body)
	if err != nil {
		panic(err)
	}
	log.Println("Body", string(body))
	var gU getUserSt
	err = json.Unmarshal(body, &gU)
	if err != nil {
		panic(err)
	}
	defer resp.Body.Close()
	log.Println("Got user with username", gU.Username, " and api key", gU.APIKey)
}

type getUserSt struct {
	UserID   string `json:"user_id"`
	APIKey   string `json:"api_key"`
	Username string `json:"username"`
}

func register() {
	var username string
	log.Println("Enter username")
	fmt.Scanln(&username)

	req, err := http.NewRequest("POST", fmt.Sprintf("%s/user", baseURL), bytes.NewBuffer([]byte(fmt.Sprintf(`{ "username": "%s"}`, username))))
	if err != nil {
		panic(err)
	}
	client := &http.Client{}
	req.Header.Set("Content-Type", "application/json")
	resp, err := client.Do(req)
	if err != nil {
		panic(err)
	}
	defer resp.Body.Close()
	body, err := io.ReadAll(resp.Body)
	if err != nil {
		panic(err)
	}
	log.Println(string(body))
	if resp.StatusCode != 200 {
		panic(fmt.Errorf("status code not 200 %d", resp.StatusCode))
	}
	var r registerst
	err = json.Unmarshal(body, &r)
	if err != nil {
		panic(err)
	}
	err = saveAPIKey(r.Apikey)
	if err != nil {
		panic(err)
	}
}
func saveAPIKey(key string) error {
	e := env{
		ApiKey: key,
	}
	body, err := json.Marshal(&e)
	if err != nil {
		return err
	}
	err = os.WriteFile("env.json", body, 0644)
	if err != nil {
		return err
	}

	return nil
}

type env struct {
	ApiKey string `json:"api-key"`
}
type registerst struct {
	Message string `json:"message"`
	Apikey  string `json:"api_key"`
}
