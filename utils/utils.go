package utils

import (
	"bytes"
	"context"
	"encoding/json"
	"errors"
	"fmt"
	"io"
	"log"
	"net/http"
	"reflect"
	"strings"
)

// Pretend there's a server here
type invoke struct {
	id      string
	payload []byte
	headers http.Header
}

const ContextKey = "bg-ctx-key"

type ContextValues struct {
	ID      string
	Headers http.Header
}

func InvokeDetails() invoke {
	i := invoke{
		id:      "ID HERE",
		payload: []byte(`{"message":"John"}`),
		headers: http.Header{},
	}
	return i
}

func Start(handler interface{}) {

	h := reflectHandler(handler)
	id := InvokeDetails()
	bg := context.Background()
	ctx := context.WithValue(bg, ContextKey, ContextValues{
		ID:      id.id,
		Headers: id.headers,
	})
	response, err := h.Invoke(ctx, []byte(`{"message":"John"}`))
	if err != nil {
		panic(err)
	}
	log.Println("Got value of ", string(response), " from function")
}

type HandlerFunc func(context.Context, []byte) (io.Reader, error)

func (h handlerFunc) Invoke(ctx context.Context, payload []byte) ([]byte, error) {
	response, err := h(ctx, payload)
	if err != nil {
		return nil, err
	}
	// if the response needs to be closed (ex: net.Conn, os.File), ensure it's closed before the next invoke to prevent a resource leak
	if response, ok := response.(io.Closer); ok {
		defer response.Close()
	}
	// optimization: if the response is a *bytes.Buffer, a copy can be eliminated
	switch response := response.(type) {
	case *jsonOutBuffer:
		return response.Bytes(), nil
	case *bytes.Buffer:
		return response.Bytes(), nil
	}
	b, err := io.ReadAll(response)
	if err != nil {
		return nil, err
	}
	return b, nil
}

type jsonOutBuffer struct {
	*bytes.Buffer
}

func reflectHandler(f interface{}) handlerFunc {
	handler := reflect.ValueOf(f)
	handlerType := reflect.TypeOf(f)
	if handlerType.Kind() != reflect.Func {
		return errorHandler(errors.New("Not a function"))
	}

	takesContext, err := handlerTakesContext(handlerType)
	if err != nil {
		return errorHandler(err)
	}
	out := &jsonOutBuffer{bytes.NewBuffer(nil)}

	return func(ctx context.Context, payload []byte) (io.Reader, error) {
		in := bytes.NewBuffer(payload)
		decoder := json.NewDecoder(in)
		encoder := json.NewEncoder(out)
		decoder.DisallowUnknownFields()

		var args []reflect.Value
		if takesContext {
			args = append(args, reflect.ValueOf(ctx))
		}
		// 1 / two values
		if handlerType.NumIn() == 1 && !takesContext || handlerType.NumIn() == 2 {
			eventType := handlerType.In(handlerType.NumIn() - 1)
			event := reflect.New(eventType)

			if err := decoder.Decode(event.Interface()); err != nil {
				return nil, err
			}
			args = append(args, event.Elem())
		}
		//}

		response := handler.Call(args)
		if len(response) > 0 {
			if errVal, ok := response[len(response)-1].Interface().(error); ok && errVal != nil {
				return nil, errVal
			}
		}
		var val interface{}
		if len(response) > 1 {
			val = response[0].Interface()
		}
		if err := encoder.Encode(val); err != nil {
			// if response is not JSON serializable, but the response type is a reader, return it as-is
			if reader, ok := val.(io.Reader); ok {
				return reader, nil
			}
			return nil, err
		}
		if reader, ok := val.(io.Reader); ok {
			// back-compat, don't return the reader if the value serialized to a non-empty json
			if strings.HasPrefix(out.String(), "{}") {
				return reader, nil
			}
		}
		return out, nil
	}
}
func handlerTakesContext(handler reflect.Type) (bool, error) {
	switch handler.NumIn() {
	case 0:
		return false, nil
	case 1:
		contextType := reflect.TypeOf((*context.Context)(nil)).Elem()
		argumentType := handler.In(0)
		if argumentType.Kind() != reflect.Interface {
			return false, nil
		}

		// handlers like func(event any) are valid.
		if argumentType.NumMethod() == 0 {
			return false, nil
		}

		if !contextType.Implements(argumentType) || !argumentType.Implements(contextType) {
			return false, fmt.Errorf("handler takes an interface, but it is not context.Context: %q", argumentType.Name())
		}
		return true, nil
	case 2:
		contextType := reflect.TypeOf((*context.Context)(nil)).Elem()
		argumentType := handler.In(0)
		if argumentType.Kind() != reflect.Interface || !contextType.Implements(argumentType) || !argumentType.Implements(contextType) {
			return false, fmt.Errorf("handler takes two arguments, but the first is not Context. got %s", argumentType.Kind())
		}
		return true, nil
	}
	return false, fmt.Errorf("handlers may not take more than two arguments, but handler takes %d", handler.NumIn())
}

type handlerFunc func(context.Context, []byte) (io.Reader, error)

func errorHandler(err error) handlerFunc {
	return func(_ context.Context, _ []byte) (io.Reader, error) {
		return nil, err
	}
}
