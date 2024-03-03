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


### Todo
#### For gRPC things use Tonic, not default thing
1 - Create logging system of sorts, gRPC system ? 
2 - Split program - Dockerise and running should be a seperate program to allow for multiple systems running later on
    ( Rest gateway to main PC, gRPC to container running machines) 
2 - Validate file names on creation
3 - Currently no validation, just assumes it'll work 

