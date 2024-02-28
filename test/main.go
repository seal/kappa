package main

import (
	"fmt"
	"github.com/seal/kappa/utils"
	"log"
)

const baseURL = "http://localhost:3000"

func main() {
	log.Println("1. Test library")
	log.Println("2. Test eps ")
	var i int
	fmt.Scanln(&i)
	switch i {
	case 1:
		TestLibrary()
	case 2:
		StartEpTest()
	}
}
func TestLibrary() {
}
