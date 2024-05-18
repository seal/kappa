package client

import (
	"bytes"
	"encoding/json"
	"errors"
	"fmt"
	"io"
	"mime/multipart"
	"net/http"
	"net/textproto"
	"os"
)

type Error string

const (
	APIKeyUnauthorized  Error = "unauthorized api key"
	Buildrequest        Error = "could not build request"
	ResponseBodyInvalid Error = "could not read body"
)

func (e Error) Error() string {
	return string(e)
}

// Client represents the client for interacting with the API.
type Client struct {
	baseURL    string
	apiKey     string
	httpClient *http.Client
}

// NewClient creates a new client with the specified base URL and API key.
func NewClient(baseURL, apiKey string) *Client {
	return &Client{
		baseURL:    baseURL,
		apiKey:     apiKey,
		httpClient: &http.Client{},
	}
}

func (c *Client) CreateContainer(language, filePath string) (*Container, error) {
	body := &bytes.Buffer{}
	writer := multipart.NewWriter(body)

	header := make(textproto.MIMEHeader)
	header.Set("API-KEY", c.apiKey)

	file, err := os.Open(filePath)
	if err != nil {
		return nil, err
	}
	defer file.Close()

	part, err := writer.CreateFormFile("file", filePath)
	if err != nil {
		return nil, err
	}
	_, err = io.Copy(part, file)
	if err != nil {
		return nil, err
	}

	err = writer.Close()
	if err != nil {
		return nil, err
	}

	req, err := http.NewRequest(http.MethodPost, fmt.Sprintf("%s/containers?language=%s", c.baseURL, language), body)
	if err != nil {
		return nil, Buildrequest
	}
	req.Header.Set("Content-Type", writer.FormDataContentType())
	req.Header.Set("API-KEY", c.apiKey)

	res, err := c.httpClient.Do(req)
	if err != nil {
		return nil, err
	}
	defer res.Body.Close()

	if res.StatusCode != http.StatusOK {
		return nil, fmt.Errorf("status code not 200: %d", res.StatusCode)
	}

	var response struct {
		ContainerID string `json:"container_id"`
		Message     string `json:"message"`
	}
	err = json.NewDecoder(res.Body).Decode(&response)
	if err != nil {
		return nil, err
	}

	container := &Container{
		Client:   c,
		ID:       response.ContainerID,
		Language: language,
	}

	return container, nil
}
func (c *Client) GetContainers() ([]Container, error) {
	req, err := http.NewRequest(http.MethodGet, fmt.Sprintf("%s/containers", c.baseURL), nil)
	if err != nil {
		return nil, Buildrequest
	}
	req.Header.Set("Content-Type", "application/json")
	req.Header.Set("API-KEY", c.apiKey)
	res, err := c.httpClient.Do(req)
	if err != nil {
		return nil, err
	}
	if res.StatusCode == 400 {
		return nil, APIKeyUnauthorized
	} else if res.StatusCode != http.StatusOK {
		return nil, errors.New(fmt.Sprintf("Unknown status code %d", res.StatusCode))
	}
	body, err := io.ReadAll(res.Body)
	if err != nil {
		return nil, ResponseBodyInvalid
	}
	var containers []Container
	err = json.Unmarshal(body, &containers)
	if err != nil {
		return nil, err
	}
	for i := range containers {
		containers[i].Client = c
	}
	return containers, nil
}

type Container struct {
	*Client
	ID       string `json:"container_id"`
	Language string `json:"language"`
}

type ContainerResponse struct {
	StatusCode int
	Body       []byte
	Headers    http.Header
}

func (c *Container) TriggerContainer(body []byte) (*ContainerResponse, error) {
	var req *http.Request
	var err error

	if body != nil {
		req, err = http.NewRequest(http.MethodPost, fmt.Sprintf("%s/trigger", c.baseURL), bytes.NewBuffer(body))
	} else {
		req, err = http.NewRequest(http.MethodPost, fmt.Sprintf("%s/trigger", c.baseURL), nil)
	}
	if err != nil {
		return nil, Buildrequest
	}

	req.Header.Set("Content-Type", "application/json")
	req.Header.Set("API-KEY", c.apiKey)
	req.Header.Set("container", c.ID)

	values := req.URL.Query()
	values.Add("container_id", c.ID)
	req.URL.RawQuery = values.Encode()

	res, err := c.httpClient.Do(req)
	if err != nil {
		return nil, err
	}
	defer res.Body.Close()

	responseBody, err := io.ReadAll(res.Body)
	if err != nil {
		// Some requests have no body
		if !errors.Is(err, io.EOF) {
			return nil, err
		}
	}

	response := &ContainerResponse{
		StatusCode: res.StatusCode,
		Body:       responseBody,
		Headers:    res.Header,
	}

	return response, nil
}
func (c *Container) Delete() error {
	req, err := http.NewRequest(http.MethodDelete, fmt.Sprintf("%s/containers", c.Client.baseURL), nil)
	if err != nil {
		return Buildrequest
	}

	values := req.URL.Query()
	values.Add("container_id", c.ID)
	req.URL.RawQuery = values.Encode()

	req.Header.Set("Content-Type", "application/json")
	req.Header.Set("API-KEY", c.Client.apiKey)

	res, err := c.Client.httpClient.Do(req)
	if err != nil {
		return err
	}
	defer res.Body.Close()

	if res.StatusCode != http.StatusOK {
		body, _ := io.ReadAll(res.Body)
		return fmt.Errorf("status code not 200: %d, body: %s", res.StatusCode, string(body))
	}

	return nil
}
