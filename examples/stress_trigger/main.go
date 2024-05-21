package main

import (
	"github.com/seal/kappa/client"
	"log"
	"math/rand"
	"sync"
	"time"
)

func RandomString(n int) string {
	var letters = []rune("abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789")

	s := make([]rune, n)
	for i := range s {
		s[i] = letters[rand.Intn(len(letters))]
	}
	return string(s)
}

const (
	concurrency = 10
	iterations  = 100 // Number of iterations per goroutine
)

func main() {
	var wg sync.WaitGroup

	// Register a new user and obtain an API key
	apiKey, err := client.CreateUser("http://localhost:3000", RandomString(20))
	if err != nil {
		log.Fatal("Error registering user:", err)
	}
	log.Printf("Registered user and got API key: %s\n", apiKey)

	// Create a new client
	c := client.NewClient("http://localhost:3000", apiKey)

	// Create a container
	container, err := c.CreateContainer(client.ContainerOptions{
		Name:     "json",
		Language: "go",
		Filepath: "../server/3_json.zip",
	})
	if err != nil {
		log.Fatal("Error creating container:", err)
	}

	// Stress test triggering the container
	stressTestTrigger(container)

	wg.Wait()
}

func stressTestTrigger(container *client.Container) {
	var wg sync.WaitGroup
	startTime := time.Now()

	for i := 0; i < concurrency; i++ {
		wg.Add(1)
		go func() {
			defer wg.Done()
			for j := 0; j < iterations; j++ {
				_, err := container.TriggerContainer([]byte(`{
    "message": "message one",
    "messageTwo": "message two"
}`))
				if err != nil {
					log.Println("Error triggering container:", err)
					continue
				}
			}
		}()
	}

	wg.Wait()
	duration := time.Since(startTime)

	log.Printf("Stress test for triggering completed in %s\n", duration)
	log.Printf("Throughput: %.2f requests/second\n", float64(concurrency*iterations)/duration.Seconds())
}
