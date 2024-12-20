# CORE ETL

Extract, transform, and load data from the Core Blockchain.

## Table of Contents

- [CORE ETL](#core-etl)
  - [Table of Contents](#table-of-contents)
  - [Introduction](#introduction)
  - [Installation](#installation)
  - [Usage](#usage)
    - [Commands](#commands)
    - [Flags](#flags)
      - [Core-etl Flags](#core-etl-flags)
      - [Export-specific Command Flags](#export-specific-command-flags)
    - [Makefile](#makefile)
      - [Makefile Commands](#makefile-commands)
    - [Docker](#docker)
      - [Build Docker Image](#build-docker-image)
      - [Run Docker Container](#run-docker-container)
    - [Docker Compose](#docker-compose)
      - [Start Services (with local node)](#start-services-with-local-node)
      - [Start Services (using remote node)](#start-services-using-remote-node)
  - [Configuration](#configuration)
    - [Environment Variables](#environment-variables)
    - [Command-Line Flags](#command-line-flags)
  - [Examples](#examples)
    - [Export Data](#export-data)
  - [Contributing](#contributing)
  - [License](#license)

## Introduction

`core-etl` is a tool designed to extract, transform, and load data from the Core Blockchain. It supports various modules and can be configured to work with different storage backends.

## Installation

To install `core-etl`, you need to have Rust and Cargo installed. You can install Rust using `rustup`:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Clone the repository:

```bash
git clone https://github.com/core-coin/core-etl.git
```

Build the binary and use it:

```bash
cargo build --release
cd target/release
./core-etl {flags}
```

Alternatively, use it inside a Docker container:

```bash
make init-libsql
make sync-local-libsql
```

## Usage

### Commands

- `export`: Export blockchain data to storage.
- `help`: Print help information.

### Flags

#### Core-etl Flags

Flag | Description | Environment Variable | Default Value
---| --- | --- | ---
`-r, --rpc-url <RPC_URL>` | URL of the RPC node that provides the blockchain data. | `RPC_URL` | wss://xcbws.coreblockchain.net
`-n, --network <NETWORK>` | Network to sync data from (e.g., mainnet, devin, private). | `NETWORK` | Mainnet
`--storage <STORAGE>` | Storage type for saving blockchain data (e.g., sqlite3, postgres). | `STORAGE` | sqlite3
`-s, --sqlite3-path <SQLITE3_PATH>` | Path to SQLite3 file where the blockchain data is saved. | `SQLITE3_PATH` | None
`-p, --postgres-db-dsn <POSTGRES_DB_DSN>` | Postgres database DSN where the blockchain data is saved. | `POSTGRES_DB_DSN` | None
`-t, --tables-prefix <TABLES_PREFIX>` | Prefix for the tables in the database. Useful when running multiple instances. | `TABLES_PREFIX` | etl
`-m, --modules <MODULES>...` | Specify which data to store (e.g., blocks, transactions, token_transfers). | `MODULES` | blocks,transactions,token_transfers
`--threads <THREADS>` | Number of working threads during the initial sync. | `THREADS` | 3
`-h, --help` | Print help information. | None | None
`-V, --version` | Print version information. | None | None

#### Export-specific Command Flags

Flag | Description | Environment Variable | Default Value
--- | --- | --- | ---
`-b, --block <BLOCK>` | Block to start syncing from. | `BLOCK` | None
`-w, --watch-tokens <WATCH_TOKENS>...` | Watch token transfers (e.g., `cbc20:token_address`). | `WATCH_TOKENS` | None
`-a, --address-filter <ADDRESS_FILTER>...` | Filter transactions by address (e.g., "0x123,0x456"). | `ADDRESS_FILTER` | None
`-r, --retention-duration <RETENTION_DURATION>` | Duration to retain data in the database. | `RETENTION_DURATION` | 0
`-c, --cleanup-interval <CLEANUP_INTERVAL>` | Interval (in seconds) for cleanup task, removing data older than retention duration. | `CLEANUP_INTERVAL` | 3600
`-l, --lazy` | Lazy mode: Do not sync while the node is syncing. Useful for slow-syncing nodes. | `LAZY` | None

### Makefile

The Makefile provides convenient commands for building, running, and managing the project.

There are commands for two storage types: Postgres and Sqlite3

Example how to setup ETL for PostgreSQL 
```bash
make build-postgres
make sync-local-postgres
```
Same for Sqlite3 (Libsql)
```bash
make build-libsql
make init-libsql
make sync-local-libsql
```

#### Makefile Commands

Instead of {storage} use postgres or libsql

Command | Description
--- | ---
`make build-{storage}` | Build the project.
`make clean-{storage}` | Clean up logs and database files.
`make init-libsql` | Initialize the database mount. Works only for sqlite3 (libsql)
`make up-{storage}` | Start services using Docker Compose.
`make down-{storage}` | Stop and remove services using Docker Compose.
`make stop-{storage}` | Stop running containers without removing them.
`make start-{storage}` | Start existing containers that were stopped.
`make sync-local-{storage}` | Sync database with a local node.
`make sync-remote-{storage}` | Sync database with a remote node.

### Docker

You can build and run the project using Docker.

#### Build Docker Image

```bash
docker build -t core-etl .
```

#### Run Docker Container

```bash
docker run -d --name core-etl -e RPC_URL=https://your.rpc.url -e SQLITE3_PATH=/path/to/your/sqlite3.db core-etl export
```

### Docker Compose

Docker Compose can be used to manage multi-container Docker applications.

#### Start Services (with local node)

```bash
make init-libsql
make sync-local-libsql
```

#### Start Services (using remote node)

```bash
make init-libsql
make sync-remote-libsql
```

## Configuration

You can configure `core-etl` using environment variables or command-line flags. Here are some examples:

### Environment Variables

```bash
export NETWORK="mainnet"
export STORAGE="sqlite3"
export SQLITE3_PATH="/path/to/your/sqlite3.db"
export TABLES_PREFIX="etl"
export MODULES="blocks,transactions,token_transfers"
```

### Command-Line Flags

```bash
./core-etl -n mainnet --storage sqlite3 -s /path/to/your/sqlite3.db -t etl -m blocks,transactions,token_transfers export
```

## Examples

### Export Data

To export blockchain data to SQLite3 storage:

```bash
./core-etl -s ./sqlite3.db export
```

Export only transaction data for the Devin network to SQLite3 storage, using 10 parallel threads for faster syncing:

```bash
./core-etl -n devin -s ./sqlite3.db -m transactions --threads 10 export
```

Export transactions and CTN transfers to Postgres with a cleanup interval of 1 hour and retention period of 24 hours:

```bash
./core-etl --storage postgres -p postgres://user:password@localhost:5432/dbname -m transactions,token_transfers export -w "ctn" -r 86400 -c 3600
```

Export blocks and transactions using a local node, with the `filtered_etl` table prefix. Do not sync data until the node is synced. Also, filter transactions by address `cb22as..21`:

```bash
./core-etl -s ./sqlite3.db -r https://127.0.0.1:8545 -t filtered_etl export -m blocks,transactions -l -a cb22as..21
```

## Contributing

Contributions are welcome! Please open an issue or submit a pull request on GitHub.

## License

This project is licensed under the CORE License. See the [LICENSE](LICENSE) file for details.
