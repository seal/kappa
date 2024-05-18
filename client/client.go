package client

import (
	"bytes"
	"encoding/json"
	"fmt"
	"io"
	"mime/multipart"
	"net/http"
	"net/textproto"
	"net/url"
	"os"
)

// Client represents the client for interacting with the API.
type Client struct {
	baseURL    string
	apiKey     string
	httpClient *http.Client
}

// ContainerOptions represents the options for creating a container.
type ContainerOptions struct {
	Language string
	Filepath string
	Name     string
}

// NewClient creates a new client with the specified base URL and API key.
func NewClient(baseURL, apiKey string) *Client {
	return &Client{
		baseURL:    baseURL,
		apiKey:     apiKey,
		httpClient: &http.Client{},
	}
}

// CreateContainer creates a new container with the provided options.
func (c *Client) CreateContainer(opts ContainerOptions) (*Container, error) {
	body := &bytes.Buffer{}
	writer := multipart.NewWriter(body)

	header := make(textproto.MIMEHeader)
	header.Set("API-KEY", c.apiKey)

	file, err := os.Open(opts.Filepath)
	if err != nil {
		return nil, fmt.Errorf("failed to open file: %w", err)
	}
	defer file.Close()

	part, err := writer.CreateFormFile("file", opts.Filepath)
	if err != nil {
		return nil, fmt.Errorf("failed to create form file: %w", err)
	}
	_, err = io.Copy(part, file)
	if err != nil {
		return nil, fmt.Errorf("failed to copy file: %w", err)
	}

	err = writer.Close()
	if err != nil {
		return nil, fmt.Errorf("failed to close writer: %w", err)
	}
	queryParams := url.Values{}
	queryParams.Add("language", opts.Language)
	queryParams.Add("name", opts.Name)

	req, err := http.NewRequest(http.MethodPost, fmt.Sprintf("%s/containers?%s", c.baseURL, queryParams.Encode()), body)
	if err != nil {
		return nil, fmt.Errorf("failed to create request: %w", err)
	}
	req.Header.Set("Content-Type", writer.FormDataContentType())
	req.Header.Set("API-KEY", c.apiKey)

	res, err := c.httpClient.Do(req)
	if err != nil {
		return nil, fmt.Errorf("failed to send request: %w", err)
	}
	defer res.Body.Close()

	if res.StatusCode != http.StatusOK {
		return nil, fmt.Errorf("unexpected status code: %d", res.StatusCode)
	}

	var response struct {
		ContainerID string `json:"container_id"`
		Message     string `json:"message"`
	}
	err = json.NewDecoder(res.Body).Decode(&response)
	if err != nil {
		return nil, fmt.Errorf("failed to decode response: %w", err)
	}

	container := &Container{
		Client:   c,
		ID:       response.ContainerID,
		Language: opts.Language,
		Name:     opts.Name,
	}

	return container, nil
}

// GetContainers retrieves a list of containers.
func (c *Client) GetContainers() ([]Container, error) {
	req, err := http.NewRequest(http.MethodGet, fmt.Sprintf("%s/containers", c.baseURL), nil)
	if err != nil {
		return nil, fmt.Errorf("failed to create request: %w", err)
	}
	req.Header.Set("Content-Type", "application/json")
	req.Header.Set("API-KEY", c.apiKey)
	res, err := c.httpClient.Do(req)
	if err != nil {
		return nil, fmt.Errorf("failed to send request: %w", err)
	}
	defer res.Body.Close()

	if res.StatusCode == http.StatusUnauthorized {
		return nil, fmt.Errorf("unauthorized API key")
	} else if res.StatusCode != http.StatusOK {
		return nil, fmt.Errorf("unexpected status code: %d", res.StatusCode)
	}
	body, err := io.ReadAll(res.Body)
	if err != nil {
		return nil, fmt.Errorf("failed to read response body: %w", err)
	}
	var containers []Container
	err = json.Unmarshal(body, &containers)
	if err != nil {
		return nil, fmt.Errorf("failed to unmarshal response: %w", err)
	}
	for i := range containers {
		containers[i].Client = c
	}
	return containers, nil
}

// Container represents a container.
type Container struct {
	*Client
	ID       string `json:"container_id"`
	Language string `json:"language"`
	Name     string `json:"name"`
}

// ContainerResponse represents the response from a container trigger.
type ContainerResponse struct {
	StatusCode int
	Body       []byte
	Headers    http.Header
}

// TriggerContainer triggers the container with the provided body.
func (c *Container) TriggerContainer(body []byte) (*ContainerResponse, error) {
	var req *http.Request
	var err error

	if body != nil {
		req, err = http.NewRequest(http.MethodPost, fmt.Sprintf("%s/trigger", c.baseURL), bytes.NewBuffer(body))
	} else {
		req, err = http.NewRequest(http.MethodPost, fmt.Sprintf("%s/trigger", c.baseURL), nil)
	}
	if err != nil {
		return nil, fmt.Errorf("failed to create request: %w", err)
	}

	req.Header.Set("Content-Type", "application/json")
	req.Header.Set("API-KEY", c.apiKey)
	req.Header.Set("container", c.ID)

	values := req.URL.Query()
	values.Add("container_id", c.ID)
	req.URL.RawQuery = values.Encode()

	res, err := c.httpClient.Do(req)
	if err != nil {
		return nil, fmt.Errorf("failed to send request: %w", err)
	}
	defer res.Body.Close()

	responseBody, err := io.ReadAll(res.Body)
	if err != nil && err != io.EOF {
		return nil, fmt.Errorf("failed to read response body: %w", err)
	}

	response := &ContainerResponse{
		StatusCode: res.StatusCode,
		Body:       responseBody,
		Headers:    res.Header,
	}

	return response, nil
}

// Delete deletes the container.
func (c *Container) Delete() error {
	req, err := http.NewRequest(http.MethodDelete, fmt.Sprintf("%s/containers", c.Client.baseURL), nil)
	if err != nil {
		return fmt.Errorf("failed to create request: %w", err)
	}

	values := req.URL.Query()
	values.Add("container_id", c.ID)
	req.URL.RawQuery = values.Encode()

	req.Header.Set("Content-Type", "application/json")
	req.Header.Set("API-KEY", c.Client.apiKey)

	res, err := c.Client.httpClient.Do(req)
	if err != nil {
		return fmt.Errorf("failed to send request: %w", err)
	}
	defer res.Body.Close()

	if res.StatusCode != http.StatusOK {
		body, _ := io.ReadAll(res.Body)
		return fmt.Errorf("unexpected status code: %d, body: %s", res.StatusCode, string(body))
	}

	return nil
}

