package main

import (
	"context"
	"encoding/json"
	"io"
	"strings"

	"github.com/seal/kappa/utils"
)

func main() {
	utils.Start(HandleRequestContext)
}

/*
No struct:
Just displays the use of ctx ( getting ID and headers)
Also displays returning and IO reader ( will not be marshalled into json)
*/
func HandleRequestContext(ctx context.Context) (io.Reader, error) {
	values := ctx.Value(utils.ContextKey).(utils.ContextValues)
	body, err := json.Marshal(values)
	if err != nil {
		return strings.NewReader(""), err
	}
	return strings.NewReader(string(body)), nil
}
