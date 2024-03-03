package main

import (
	"context"
	"github.com/seal/kappa/utils"
)

func main() {
	utils.Start(HandleRequestString)
}

/*
String input, will return "here" ( json marshals so "" is included
*/
func HandleRequestString(ctx context.Context) (string, error) {
	return "here", nil
}
