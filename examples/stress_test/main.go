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
	numClients  = 1 // Number of clients
	concurrency = 10
	iterations  = 10 // Number of iterations per goroutine
)

func main() {
	var wg sync.WaitGroup
	clientAPIs := make([]string, numClients)

	// Register new users and obtain API keys
	for i := 0; i < numClients; i++ {
		apiKey, err := client.CreateUser("http://localhost:3000", RandomString(20))
		if err != nil {
			log.Fatal("Error registering user:", err)
		}
		clientAPIs[i] = apiKey
		log.Printf("Registered user%d and got API key: %s\n", i, apiKey)
	}

	// Stress test each client
	for i := 0; i < numClients; i++ {
		wg.Add(1)
		go func(clientIndex int) {
			defer wg.Done()
			c := client.NewClient("http://localhost:3000", clientAPIs[clientIndex])

			// Stress test CreateContainer and TriggerContainer endpoints
			stressTestCreateAndTrigger(c, "3_json.zip", "json", "go", []byte(`{
    "message": "message one",
    "messageTwo": "message two"
}`))
			stressTestCreateAndTrigger(c, "2_context.zip", "context", "go", nil)
			stressTestCreateAndTrigger(c, "1_string.zip", "string", "go", nil)

			// Stress test GetContainers and Delete endpoints
			stressTestGetAndDelete(c)
		}(i)
	}

	wg.Wait()
}

func stressTestCreateAndTrigger(c *client.Client, filepath, name, language string, body []byte) {
	var wg sync.WaitGroup
	startTime := time.Now()

	for i := 0; i < concurrency; i++ {
		wg.Add(1)
		go func() {
			defer wg.Done()
			for j := 0; j < iterations; j++ {
				container, err := c.CreateContainer(client.ContainerOptions{
					Name:     name,
					Language: language,
					Filepath: "../server/" + filepath,
				})
				if err != nil {
					log.Println("Error creating container:", err)
					continue
				}

				_, err = container.TriggerContainer(body)
				if err != nil {
					log.Println("Error triggering container:", err)
					continue
				}

			}
		}()
	}

	wg.Wait()
	duration := time.Since(startTime)

	log.Printf("Stress test for %s completed in %s\n", filepath, duration)
	log.Printf("Throughput: %.2f requests/second\n", float64(concurrency*iterations)/duration.Seconds())
}

func stressTestGetAndDelete(c *client.Client) {
	var wg sync.WaitGroup
	time.Sleep(10 * time.Second)
	startTime := time.Now()

	containers, err := c.GetContainers()
	if err != nil {
		log.Println("Error getting containers:", err)
		return
	}

	for _, container := range containers {
		err := container.Delete()
		if err != nil {
			log.Println("Error deleting container:", err)
			continue
		}
	}

	wg.Wait()
	duration := time.Since(startTime)

	log.Printf("Stress test for GetContainers and Delete completed in %s\n", duration)
}
