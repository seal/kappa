package main

import (
	"log"

	"github.com/seal/kappa/client"
)

func main() {
	client := client.NewClient("http://localhost:3000", "e1f11936-2004-46ee-b310-84ddb8fb8d14")
	log.Println("Getting containers")
	container, err := client.CreateContainer("go", "../../test/tests/3_json.zip")
	if err != nil {
		panic(err)
	}
	log.Printf("Created container with ID: %s\n", container.ID)
	body := []byte(`{
    "message": "message one",
    "messageTwo": "message two"
}`)

	resp, err := container.TriggerContainer(body)
	if err != nil {
		panic(err)
	}
	log.Println("---------------------")
	log.Println("Triggered container with ID: ", container.ID)
	log.Println("Status Code:", resp.StatusCode)
	log.Println("Body:", string(resp.Body))
	log.Println("Headers:")
	for k, v := range resp.Headers {
		log.Println(k, v)
	}
	log.Println("--------------------")
	log.Println("Deleting all containers now ")

	containers, err := client.GetContainers()
	if err != nil {
		panic(err)
	}
	for _, container := range containers {
		log.Println("Deleting ID", container.ID, " - with language ", container.Language)
		err := container.Delete()
		if err != nil {
			panic(err)
		}
	}
}
