This repository contains the code for my implementation of AWS lambda written in Rust & GoLang

** This project is incomplete and currently in development*

## Create a function
View example-main.go to get the idea of creating a function 

## Running the code
### Postgres
```bash
docker-compose up -d 
```
### Rust 
```bash
cargo run 
```
Http server will be created on port 3000 



## Functionality 
Run the go program in test/ for testing functionality 
This includes create user, container, run container etc


#### For gRPC things use Tonic, not default package
1 - Create logging system of sorts, gRPC system ? 

2 - Split program - Dockerise and running should be a seperate program to allow for multiple systems running later on
    ( Rest gateway to main PC, gRPC to container running machines) 
    (Create two bin's, client & server essentially)

3 - Validate file names on creation, main.go etc?

4 - Currently no validation, just assumes it'll work 

5 - Add proper logging no eprintln in docker functions 
^^ .... gRPC ? 


### Tests
All tests are written in Go, to test rust http functionality 
Re-write in Rust at later date.

