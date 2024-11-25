# core-etl

Extract, transform, and load Core Blockchain data.

## Table of Contents

- [Introduction](#introduction)
- [Installation](#installation)
- [Usage](#usage)
  - [Commands](#commands)
  - [Flags](#flags)
  - [Makefile](#makefile)
  - [Docker](#docker)
  - [Docker Compose](#docker-compose)
- [Configuration](#configuration)
- [Examples](#examples)
- [Contributing](#contributing)
- [License](#license)

## Introduction

`core-etl` is a tool designed to extract, transform, and load data from the Core Blockchain. It supports various modules and can be configured to work with different storage backends.

## Local Installation

To install `core-etl`, you need to have Rust and Cargo installed. You can install Rust using `rustup`:

```
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Clone the repository:

```
git clone https://github.com/core-coin/core-etl.git
```

Now you can build binary and use it

```
cargo build --release
cd target/release
./core-etl {flags}
```
or use it inside Docker container
```
make init
make sync-local
```

## Usage

### Commands

- `export`: Export blockchain data to storage.
- `help`: Print help information.

### Flags

#### Core-etl Flags

| Flag | Description | Environment Variable | Default Value |
|------|-------------|----------------------|---------------|
| `-r, --rpc-url <RPC_URL>` | URL of the RPC node that provides the blockchain data. | `RPC_URL` | wss://xcbws.coreblockchain.net |
| `-n, --network <NETWORK>` | Network to sync data from (e.g., mainnet, devin, private). If flag is set, `rpc_url` is not required. | `NETWORK` | Mainnet |
| `--storage <STORAGE>` | Storage type used for saving the blockchain data. Possible values: `sqlite3-storage`, `xata-storage` | `STORAGE` | `sqlite3-storage` |
| `-s, --sqlite3-path <SQLITE3_PATH>` | Path to SQLite3 file where the blockchain data is saved. | `SQLITE3_PATH` | None |
| `-x, --xata-db-dsn <XATA_DB_DSN>` | Xata database DSN where the blockchain data is saved. | `XATA_DB_DSN` | None |
| `-t, --tables-prefix <TABLES_PREFIX>` | Prefix for the tables in the database. Useful when running multiple instances of the ETL. | `TABLES_PREFIX` | `etl` |
| `-m, --modules <MODULES>...` | Which data will be stored in the database. | `MODULES` | `blocks,transactions,token_transfers` |
| `-h, --help` | Print help information. | None | None |
| `-V, --version` | Print version information. | None | None |

#### Export-specified Command Flags

| Flag | Description | Environment Variable | Default Value |
|------|-------------|----------------------|---------------|
| `-b, --block <BLOCK>` | Block to start syncing from. | `BLOCK` | None |
| `-w, --watch-tokens <WATCH_TOKENS>...` | Watch token transfers. Provide a token type and address to watch in the format: "token_type:token_address,token_type:token_address". Example: "cbc20:cb19c7acc4c292d2943ba23c2eaa5d9c5a6652a8710c" - to watch Core Token transfers. |`WATCH_TOKENS` | None |
| `-a, --address-filter <ADDRESS_FILTER>...` | Filter transactions by address. Provide a list of addresses to filter. Example: "0x123,0x456,0x789". |`ADDRESS_FILTER` | None |
| `-r, --retention-duration <RETENTION_DURATION>` | How long to retain data in the database. |`RETENTION_DURATION` | `0` |
| `-c, --cleanup-interval <CLEANUP_INTERVAL>` | How often to run the cleanup task. Value is in seconds. Cleanup task will remove data older than retention_duration. |`CLEANUP_INTERVAL` | `3600` |
| `-l, --lazy` | Lazy mode. Do not sync while node is syncing This is useful for nodes that take a long time to sync . |`LAZY` | None |

### Makefile

The Makefile provides convenient commands for building, running, and managing the project.

#### Commands

| Command | Description |
|---------|-------------|
| `make build` | Build the project. |
| `make clean` | Clean up log and database files. |
| `make init` | Initialize the database mount. |
| `make up` | Start the services using Docker Compose. |
| `make down` | Stop and remove the services using Docker Compose. |
| `make stop` | Stop running containers without removing them. |
| `make start` | Start existing containers that were stopped. |
| `make sync-local` | Start syncing sqlite database and running libsql-server in read-only mode with local node. |
| `make sync-remote` | Start syncing sqlite database and running libsql-server in read-only mode using remote node. |


### Docker

You can build and run the project using Docker.

#### Build Docker Image

```
docker build -t core-etl .
```

#### Run Docker Container

```
docker run -d --name core-etl -e RPC_URL=https://your.rpc.url -e SQLITE3_PATH=/path/to/your/sqlite3.db core-etl export
```

### Docker Compose

Docker Compose can be used to manage multi-container Docker applications.

#### Start Services (with local node)

```
make init
make sync-local
```

#### Start Services (using remote node)

```
make init
make sync-remote
```

## Configuration

You can configure `core-etl` using environment variables or command-line flags. Here are some examples:

### Environment Variables

```
export NETWORK="mainnet"
export STORAGE="sqlite3-storage"
export SQLITE3_PATH="/path/to/your/sqlite3.db"
export TABLES_PREFIX="etl"
export MODULES="blocks,transactions,token_transfers"
```

### Command-Line Flags

```
./core-etl -n mainnet --storage sqlite3-storage -s /path/to/your/sqlite3.db -t etl -m blocks,transactions,token_transfers export
```

## Examples

### Export Data

To export blockchain data to SQLite3 storage:

```
./core-etl -s ./sqlite3.db export
```

Export only transaction data for Devin network to SQLite3 storage

```
./core-etl -n devin -s ./sqlite3.db -m transactions export
```

Export transactions and CTN transfers to Postgres with cleanup interval 1h and retention period 24h
 ```
./core-etl --storage xata-storage -x postgres://user:password@localhost:5432/dbname -m transactions,token_transfers export -w "ctn" -r 86400 -c 3600
```

Export blocks and transactions using local node. Use `filtered_etl` prefix for tables. Do not sync data in sqlite until node will be synced. Also filter transactions by address `cb22as..21`
```
./core-etl -s ./sqlite3.db -r https://127.0.0.1:8545 -t filtered_etl export -m blocks,transactions -l -a cb22as..21
```


## Contributing

Contributions are welcome! Please open an issue or submit a pull request on GitHub.

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.