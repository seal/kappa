package main

import (
	"fmt"
	"log"
)

const baseURL = "http://localhost:3000"
const containerURL = "http://localhost:5182"

func main() {
	log.Println("Please enter an option")
	log.Println("1. Test Rust")
	log.Println("2. Test Container")
	var i int
	fmt.Scanln(&i)
	switch i {
	case 1:
		StartEpTest()
	case 2:
		StartContainerTest()
	}
}
