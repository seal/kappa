# Kappa - AWS Lambda Implementation in Rust & GoLang

This repository contains the code for my implementation of AWS Lambda written in Rust and GoLang.

**Note: This project is currently in development and may be incomplete.**

## Table of Contents
- [Getting Started](#getting-started)
  - [Prerequisites](#prerequisites)
  - [Installation](#installation)
- [Usage](#usage)
  - [Creating a Function](#creating-a-function)
  - [Running the Code](#running-the-code)
    - [Postgres](#postgres)
    - [Rust](#rust)
- [Client Example](#client-example)
- [Server Example](#server-example)
- [Functionality](#functionality)
- [Contributing](#contributing)
- [License](#license)

## Getting Started

These instructions will help you set up the project on your local machine for development and testing purposes.

### Prerequisites

Make sure you have the following installed:
- [Rust](https://www.rust-lang.org/tools/install)
- [GoLang](https://golang.org/doc/install)
- [Docker](https://www.docker.com/get-started)

### Installation

1. Clone the repository:
   ```bash
   git clone https://github.com/seal/kappa.git
   ```

2. Change to the project directory:
   ```bash
   cd kappa
   ```

## Usage
### Registering a user 
```go
	api_key, err := client.CreateUser("http://localhost:3000", "username_here")
```

### Creating a Function

View /examples/server/* for an example 

Then zip via:
```
zip -r my_folder.zip my_folder/*
```

### Uploading function

```go
	container3, err := c.CreateContainer(client.ContainerOptions{
		Name:     "...namehere",
		Language: "go", // Currently only go supported
		Filepath: "filePath to .zip here",
	})

```

### Running the Code

#### Postgres
Start the Postgres database using Docker Compose:
```bash
docker-compose up -d
```

#### Rust
Run the Rust code:
```bash
cargo run
```
The HTTP server will be created on port 3000.


## Server Example

Here's an example of how to implement the server:

```go
// examples/client/main.go
package main

```

```go
//examples/server/main.go
func main() {
    utils.Start(FuncName)
}

// HandleRequestContext returns an IO reader response
func FuncName(ctx context.Context) (io.Reader, error) {
    // Code omitted
    return strings.NewReader(body), nil
}

```
### Other types

Possible parameter types are:

1:

```go
func FuncName(ctx context.Context) (string, error){
	values := ctx.Value(utils.ContextKey).(utils.ContextValues)
    /*
type ContextValues struct {
	ID      string      `json:"id"`
	Headers http.Header `json:"headers"`
}
*/
}
```

2:

```go
func FuncName(ctx context.Context, event MyEvent) (Response, error) {
	return Response{MessageOne: event.Message, MessageTwo: event.MessageTwo}, nil
}

type Response struct {
	MessageOne string `json:"messageOne"`
	MessageTwo string `json:"messagewTwo"`
}
type MyEvent struct {
	Message    string `json:"message"`
	MessageTwo string `json:"messageTwo"`
}
```

Structs can be customized to the users need etc


## Functionality

Run the /examples/client/main.go for examples

## Contributing

Contributions are welcome! If you find any issues or have suggestions for improvements, please open an issue or submit a pull request.

## License

This project is licensed under the [MIT License](LICENSE).
