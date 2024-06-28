package utils

import (
	"bytes"
	"context"
	"errors"
	"fmt"
	"io"
	"log"
	"net/http"
	"reflect"
	"strings"
	"time"

	"github.com/gorilla/mux"
	jsoniter "github.com/json-iterator/go"
)

type invoke struct {
	id      string
	payload []byte
	headers http.Header
}

const ContextKey = "bg-ctx-key"

type ContextValues struct {
	ID      string      `json:"id"`
	Headers http.Header `json:"headers"`
}

var json = jsoniter.ConfigCompatibleWithStandardLibrary

// MarshalJSON custom marshaller for ContextValues
func (cv ContextValues) MarshalJSON() ([]byte, error) {
	headers := make(map[string][]string)
	for key, values := range cv.Headers {
		headers[key] = values
	}
	return json.Marshal(struct {
		ID      string              `json:"id"`
		Headers map[string][]string `json:"headers"`
	}{
		ID:      cv.ID,
		Headers: headers,
	})
}

func invokeDetails(r *http.Request, b []byte) invoke {
	i := invoke{
		id:      r.URL.Query().Get("container_id"),
		payload: b,
		headers: r.Header.Clone(),
	}
	return i
}
func Start(handler interface{}) {

	r := mux.NewRouter()
	r.HandleFunc("/", func(w http.ResponseWriter, r *http.Request) {
		body, err := io.ReadAll(r.Body)
		if err != nil {
			w.WriteHeader(http.StatusInternalServerError)
			w.Write([]byte(`{
                "error":"` + err.Error() + `"
            }`))
			return
		}
		defer r.Body.Close()
		newInvoke := invokeDetails(r, body)
		h := reflectHandler(handler)
		response, err := startDetails(h, newInvoke)
		if err != nil {
			w.WriteHeader(http.StatusInternalServerError)
			w.Write([]byte(`{
                "error":"` + err.Error() + `"
            }`))
			return
		}
		w.WriteHeader(http.StatusOK)
		w.Write(response)
	})
	http.Handle("/", r)
	srv := &http.Server{
		Handler: r,
		Addr:    "0.0.0.0:5182",
		// customers job to fix their server :)
		WriteTimeout: 1000 * time.Second,
		ReadTimeout:  1000 * time.Second,
	}

	log.Fatal(srv.ListenAndServe())

}
func startDetails(h handlerFunc, id invoke) ([]byte, error) {
	bg := context.Background()
	ctx := context.WithValue(bg, ContextKey, ContextValues{
		ID:      id.id,
		Headers: id.headers,
	})
	response, err := h.Invoke(ctx, id.payload)
	return response, err

}

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

// For readability
// type handlerFunc func(context.Context, []byte) (io.Reader, error)
func reflectHandler(f interface{}) handlerFunc {
	handler := reflect.ValueOf(f)
	handlerType := reflect.TypeOf(f)
	// If not function, return, remember we need a function that is called
	if handlerType.Kind() != reflect.Func {
		return errorHandler(errors.New("Not a function"))
	}

	// If it takes *context* ( id, headers etc)
	takesContext, err := handlerTakesContext(handlerType)
	if err != nil {
		return errorHandler(err)
	}
	// Create a buffer to store outputs of function
	out := &jsonOutBuffer{bytes.NewBuffer(nil)}
	// Return a common form of function
	return func(ctx context.Context, payload []byte) (io.Reader, error) {
		// Decoder for stuff coming in
		in := bytes.NewBuffer(payload)
		decoder := json.NewDecoder(in)
		//Encoder for stuff going out
		encoder := json.NewEncoder(out)
		// This means *extra* fields will cause an error
		/*

			type Person struct {
			    Name string `json:"name"`
			    Age  int    `json:"age"`
			}
			with  json : {"name": "John", "age": 30, "city": "New York"}
			will cause an error
		*/
		decoder.DisallowUnknownFields()
		// Create the arguments that we will "call" the function with
		var args []reflect.Value
		// If the first arg is context, append it
		if takesContext {
			args = append(args, reflect.ValueOf(ctx))
		}
		// 1 / two values
		// If 1 value ( no context) or two values,
		if (handlerType.NumIn() == 1 && !takesContext) || handlerType.NumIn() == 2 {
			// Create instance of handler type
			eventType := handlerType.In(handlerType.NumIn() - 1)
			event := reflect.New(eventType)
			// Decode the type into the json buffer
			if err := decoder.Decode(event.Interface()); err != nil {
				return nil, err
			}
			// Append arg
			args = append(args, event.Elem())
		}
		// Prepare the response
		response := handler.Call(args)
		if len(response) > 0 {
			// If there's response, and there's an error, return it
			if errVal, ok := response[len(response)-1].Interface().(error); ok && errVal != nil {
				return nil, errVal
			}
		}
		// If there's a value returned, create an interface from it and cast response to it
		var val interface{}
		if len(response) > 1 {
			val = response[0].Interface()
		}
		// Encode the value with the json encoder
		if err := encoder.Encode(val); err != nil {
			// if response is not JSON serializable, but the response type is a reader, return it as-is
			if reader, ok := val.(io.Reader); ok {
				return reader, nil
			}
			//not json serializable, and not a reader, aka error
			return nil, err
		}
		// If it's "encoded" an empty reader, return the reader not the {}
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
