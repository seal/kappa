package main

import (
	"log"
	"time"

	"github.com/seal/kappa/client"
)

func main() {
	// Register a new user via:
	//api_key, err := client.CreateUser("http://localhost:3000", "username_here")
	//if err != nil {
	//Handle error
	//} else {
	//log.Println("registered and got api key", api_key)
	//}
	c := client.NewClient("http://localhost:3000", "85ebefa0-dfe7-4c3a-a545-313664d64f8e")

	// 3_json.zip
	log.Println("Getting containers for 3_json.zip")
	container3, err := c.CreateContainer(client.ContainerOptions{
		Name:     "json",
		Language: "go",
		Filepath: "../server/3_json.zip",
	})
	if err != nil {
		panic(err)
	}
	log.Printf("Created container with ID: %s\n", container3.ID)
	body3 := []byte(`{
    "message": "message one",
    "messageTwo": "message two"
}`)
	time3 := time.Now()
	resp3, err := container3.TriggerContainer(body3)
	if err != nil {
		panic(err)
	}
	log.Println("---------------------")
	log.Println("Triggered container with ID: ", container3.ID)
	log.Println("Status Code:", resp3.StatusCode)
	log.Println("Body:", string(resp3.Body))
	log.Println("Headers:")
	for k, v := range resp3.Headers {
		log.Println(k, v)
	}
	log.Println("Time taken:")
	log.Println(time.Since(time3))
	log.Println("--------------------")

	// 2_context.zip
	log.Println("Getting containers for 2_context.zip")
	container2, err := c.CreateContainer(client.ContainerOptions{
		Name:     "context",
		Language: "go",
		Filepath: "../server/2_context.zip",
	})
	if err != nil {
		panic(err)
	}
	log.Printf("Created container with ID: %s\n", container2.ID)
	time2 := time.Now()
	resp2, err := container2.TriggerContainer(nil)
	if err != nil {
		panic(err)
	}
	log.Println("---------------------")
	log.Println("Triggered container with ID: ", container2.ID)
	log.Println("Status Code:", resp2.StatusCode)
	log.Println("Body:", string(resp2.Body))
	log.Println("Headers:")
	for k, v := range resp2.Headers {
		log.Println(k, v)
	}

	log.Println("Time taken:")
	log.Println(time.Since(time2))
	log.Println("--------------------")

	// 1_string.zip
	log.Println("Getting containers for 1_string.zip")
	container1, err := c.CreateContainer(client.ContainerOptions{
		Name:     "string",
		Language: "go",
		Filepath: "../server/1_string.zip",
	})
	if err != nil {
		panic(err)
	}
	log.Printf("Created container with ID: %s\n", container1.ID)
	time1 := time.Now()
	resp1, err := container1.TriggerContainer(nil)
	if err != nil {
		panic(err)
	}
	log.Println("---------------------")
	log.Println("Triggered container with ID: ", container1.ID)
	log.Println("Status Code:", resp1.StatusCode)
	log.Println("Body:", string(resp1.Body))
	log.Println("Headers:")
	for k, v := range resp1.Headers {
		log.Println(k, v)
	}

	log.Println("Time taken:")
	log.Println(time.Since(time1))
	log.Println("--------------------")

	log.Println("Deleting all containers now ")

	containers, err := c.GetContainers()
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
