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

### Creating a Function

View /examples/server/* for an example 

Then zip via:
```
zip -r my_folder.zip my_folder/*
```
Upload via GoLang code

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

## Client Example

Here's an example of how to use the client:

```go
// examples/client/main.go
package main

// ... (client code omitted for brevity) ...

func main() {
    c := client.NewClient("http://localhost:3000", "e1f11936-2004-46ee-b310-84ddb8fb8d14")

    // Create and trigger containers for different ZIP files
    // ... (code omitted for brevity) ...

    // Delete all containers
    containers, err := c.GetContainers()
    if err != nil {
        panic(err)
    }
    for _, container := range containers {
        log.Println("Deleting ID", container.ID, " - with language ", container.Language)
        err := container.Delete()
        if err != nil {
            panic(err)
        }
    }
}
```

## Server Example

Here's an example of how to implement the server:

```go
// examples/client/main.go
package main

```

```go
//examples/server/main.go
func main() {
    utils.Start(HandleRequestContext)
    //	utils.Start(HandleRequestString)
    //utils.Start(HandleRequestEvent)
}

// HandleRequestString returns a string response
func HandleRequestString(ctx context.Context) (string, error) {
    return "here", nil
}

// HandleRequestContext returns an IO reader response
func HandleRequestContext(ctx context.Context) (io.Reader, error) {
    // ... (code omitted for brevity) ...
}

// HandleRequestEvent handles a custom event struct
func HandleRequestEvent(ctx context.Context, event MyEvent) (Response, error) {
    return Response{MessageOne: event.Message, MessageTwo: event.MessageTwo}, nil
}
```

## Functionality

Run the /examples/client/main.go for examples

## Contributing

Contributions are welcome! If you find any issues or have suggestions for improvements, please open an issue or submit a pull request.

## License

This project is licensed under the [MIT License](LICENSE).
