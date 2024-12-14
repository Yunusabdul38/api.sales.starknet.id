# api.sales.starknet.id

## Overview

This monorepo contains three programs to manage the sales data of the StarkNetID naming smart contract in a secure and privacy preserving way.

## Prerequisites

### Install Rust

To run the project without issues you need to have a Rust version >= 1.76.0. To check your rust version run the following command in a terminal.

```bash
rustc --version
```

If you don't have Rust installed, please go to the [Rust installation page](https://doc.rust-lang.org/book/ch01-01-installation.html) for further instructions.

### Install Deno

Install Deno for the indexer componen. To install deno for the indexer run this command on your powershell

```powershell
irm https://deno.land/install.ps1 | iex
```

### Install Git

Go to the [Git installation page](https://git-scm.com/downloads) and follow the instructions for your operating system to install Git.

### Install Docker

To run the database a Docker container is necessary, you need to have Docker engine version >= 3.6 To check your Docker engine version run the following command in a terminal.

```bash
docker --version
```

If you don't have Docker installed, please go to the [Docker installation page](https://docs.docker.com/get-started/get-docker/) for further instructions.

## Installation Instructions

Fork the repository and clone the forked repository to your local system

```bash
git clone https://github.com/<your-user>/api.sales.starknet.id.git
cd api.sales.starknet.id
```

To build the project use the following command in a terminal

```bash
cargo build
```

The command above will run `cargo build` with the `--debug` flag, which compiles faster, includes debug symbols for easier debugging. However it produces a larger binary, for development purposes the command above is fine.

If you wish to create an optimized binary without debug information run the following command in a terminal

```bash
cargo build --release
```

Cache dependencies for the indexer

```bash
cd indexer
deno cache --reload
```

## Running the Project

To run the project successfully you'll need to do the following steps:
1.Deploy `db-docker-compose.yml` file to use MongoDB database.
Once inside the directory of the project, you need to run the following command:

```bash
docker-compose -f db-docker-compose.yml up -d
```

The command above will create a container running the MongoDB database, however the information you add to the database isn't persistent, you'll need to modify the db-docker-compose.yml file to include a volume. For more information regarding Docker-compose files and volumes go the this [page](https://docs.docker.com/engine/storage/volumes/).

2. Create `config.toml` file using the `config.template.toml` file.
Create a `config.toml` file by copying and modifying the `config.template.toml` file. Make sure you update the following fields as required to run the project successfully:

- `connection_string`, this is the string to connect to the database. If the `db-docker-compose.yml` isn't changed the connection string would be: `mongodb://quests:password@localhost:27017`
- `secret_key`, this is the secret used for the JWT token. You can change it or leave as is.
- `expiry_duration`, this is the expiry duration of the JWT token. You should change it according to your needs the time is stored in miliseconds.
- `rpc_url`, this is to interact with the blockchain you can use a public RPC such as [Lava](https://www.lavanet.xyz/get-started/starknet) or a private node provider such as [Alchemy](https://www.alchemy.com) or [Infura](https://www.infura.io). Alchemy and Infura require an account to get a private RPC, while Lava is completely public.
- In the section of `[watchtower]`, set `enabled` to false. If you wish to setup the watchtower correctly, you can check the Watchtower repositories for further information. [Watchtower frontend](https://github.com/starknet-id/watchtower.starknet.id) and [Watchtower backend](https://github.com/starknet-id/watchtower_server) 

## Run the Components

### 1. API Endpoint (`api_endpoint`)

To run the API endpoint read and follow the instructions below

- **Language**: Rust
- **Function**: API for frontend to register sales with metadata (user email, tax state, etc.)
- **Directory**: `./api_endpoint`

## Open terminal

```bash
cd api_endpoint
cargo run
```

### 2. Indexer (`indexer`)

To run the Indexer Read and follow the instructions below

- **Language**: Deno
- **Function**: Fetches transactions and associated metadata hashes.
- **Directory**: `./indexer`
- **Note**: A free token for the indexer can be generated on Apibara. Visit [Apibara's website](https://www.apibara.com/) for more details.

## Open a Second terminal and execute this command

```bash
cd indexer
deno run --allow-net index.ts
```

### 3. Sale Actions (`sale_actions`)

- **Language**: Rust
- **Function**: Automates actions like sending or scheduling emails upon a sale.
- **Directory**: `./sale_actions`

## Open the third terminal to run the sales action

```bash
cd sale_actions
cargo run
```

## Troubleshooting

If your expected output doesn't includes the following text:
```bash
database: connected
server: listening on http://0.0.0.0:8080
```
This means you didn't run the Docker container which runs the database. To fix this, you'll need to run the Docker container with the command mentioned in the first step of the section Running the Project.

```bash
docker-compose -f db-docker-compose.yml up -d
```

If you get the following output:

```bash
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 59.57s
     Running `target/debug/quest_server`
quest_server: starting v0.1.0
thread 'main' panicked at src/config.rs:212:9:
error: unable to read file with path "config.toml"
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

This means you didn't create the `config.toml` file. To fix this, you'll need to create the `config.toml` file with the steps mentioned in the second step of the section Running the Project.

If you get the following output:

```bash
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.55s
     Running `target/debug/quest_server`
quest_server: starting v0.1.0
thread 'main' panicked at src/main.rs:34:49:
called `Result::unwrap()` on an `Err` value: RelativeUrlWithoutBase
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

This means you didn't add the `rpc_url` in the `config.toml` file. To fix this, you'll need to add the `rpc_url` to the `config.toml` file. Please refer the second step of the section Running the Project for further instructions on how to add the `rpc_url`.

If you get the following output:

```bash
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.49s
     Running `target/debug/quest_server`
quest_server: starting v0.1.0
thread 'main' panicked at src/main.rs:29:10:
called `Result::unwrap()` on an `Err` value: Error { kind: InvalidArgument { message: "connection string contains no scheme" }, labels: {}, wire_version: None, source: None }
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

This means you didn't add the `connection_string` in the `config.toml` file. To fix this, you'll need to add the `connection_string` to the `config.toml` file. Please refer to the second step of the section Running the Project for further instructions on how to add the `connection_string`.

If you get the following output:

```bash
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 5.41s
     Running `target\debug\quest_server.exe`
quest_server: starting v0.1.0
thread 'main' panicked at src\config.rs:218:13:
error: unable to deserialize config. newline in string found at line 6 column 63
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
error: process didn't exit successfully: `target\debug\quest_server.exe` (exit code: 101)
```

This means you probably forgot the following character in the `config.toml` file: ". To fix this, you'll need to check that the fields you modified while creating the `config.toml` file have their opening and closing character. As an example, it should look like this:

`connection_string = "mongodb://quests:password@localhost:27017"`

and NOT like this

`connection_string = "mongodb://quests:password@localhost:27017`

If you get the following output:

```bash
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.53s
     Running `target/debug/quest_server`
INFO: quest_server: starting v0.1.0
Failed to post log: "Invalid token or token expired"
```

This means that you didn't setup the credentials for Watchtower. To fix this, you'll need to set the `enabled` field in `[watchtower]` to false in the `config.toml` file. Please refer the second step of the section Running the Project for further instructions if you wish to keep the `[watchtower]` enabled.

Rust Version Mismatch:
If you encounter errors related to Rust, ensure your version meets the requirement (>= 1.76.0)

## Contributors ✨

Thanks goes to these wonderful people (emoji key)

This project follows the [all-contributors](https://github.com/all-contributors/all-contributors) specification. Contributions of any kind welcome!
