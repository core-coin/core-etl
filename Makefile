# Variables
CORE_ETL_FLAGS ?= -s /libsql_data/data.sqld/dbs/default/data
CORE_ETL_EXPORT_FLAGS ?= -w="ctn" -l
GO_CORE_FLAGS ?= --ws --ws.addr 0.0.0.0 --syncmode fast --cache=128 --snapshot=false
CARGO := cargo

# Targets
.PHONY: build-libsql up-libsql down-libsql clean-libsql init-libsql sync-local-libsql sync-remote-libsql stop-libsql start-libsql clean-volume-libsql
.PHONY: build-postgres up-postgres down-postgres clean-volume-postgres init-postgres sync-local-postgres sync-remote-postgres stop-postgres start-postgres
.PHONY: build run run-debug test clean fmt clippy

######################################### LIBSQL #########################################

# Build Docker images
build-libsql:
	docker-compose -f ./docker-compose/docker-compose-local-libsql.yml build

# Initialize database mount that is used both by core-etl and libsql-server
init-libsql:
	@echo "Initializing database mount"
	docker-compose -f ./docker-compose/docker-compose-init-libsql.yml up  -d
	@sleep 3
	docker-compose -f ./docker-compose/docker-compose-init-libsql.yml down
# Builds, (re)creates, starts, and attaches to containers for a service
# This runs the additional go-core container 
sync-local-libsql:
	GO_CORE_FLAGS="$(GO_CORE_FLAGS)" CORE_ETL_FLAGS="-r ws://go-core:8546 $(CORE_ETL_FLAGS)" CORE_ETL_EXPORT_FLAGS="$(CORE_ETL_EXPORT_FLAGS)" docker-compose -f ./docker-compose/docker-compose-local-libsql.yml up -d

# Builds, (re)creates, starts, and attaches to containers for a service
# This runs the additional go-core container 
sync-remote-libsql:
	CORE_ETL_FLAGS="-n mainnet $(CORE_ETL_FLAGS)" CORE_ETL_EXPORT_FLAGS="$(CORE_ETL_EXPORT_FLAGS)" docker-compose -f ./docker-compose/docker-compose-remote-libsql.yml up -d

# Stops and removes containers, networks, volumes, and images created by docker-compose up
down-libsql:
	docker-compose -f ./docker-compose/docker-compose-local-libsql.yml down

# Stops running containers without removing them. The containers can be restarted with make start
stop-libsql:
	docker-compose -f ./docker-compose/docker-compose-local-libsql.yml stop

# Starts existing containers that were stopped
start-libsql:
	docker-compose -f ./docker-compose/docker-compose-local-libsql.yml start

# Clean shared volume
clean-volume-libsql:
	@echo "Cleaning up libsql database volume"
	@sudo rm -rf libsql_data


######################################### POSTGRESQL #########################################

# Build Docker images
build-postgres:
	docker-compose -f ./docker-compose/docker-compose-local-postgres.yml build

# Builds, (re)creates, starts, and attaches to containers for a service
# This runs the additional go-core container 
sync-local-postgres:
	GO_CORE_FLAGS="$(GO_CORE_FLAGS)" CORE_ETL_FLAGS="-r ws://localhost:8546 --storage postgres -p postgres://etl_user:etl_password@localhost:5432/etl_database" CORE_ETL_EXPORT_FLAGS="$(CORE_ETL_EXPORT_FLAGS)" docker-compose -f ./docker-compose/docker-compose-local-postgres.yml up -d

# Builds, (re)creates, starts, and attaches to containers for a service
# This runs the additional go-core container 
sync-remote-postgres:
	CORE_ETL_FLAGS="-n mainnet --storage postgres -p postgres://etl_user:etl_password@localhost:5432/etl_database" CORE_ETL_EXPORT_FLAGS="$(CORE_ETL_EXPORT_FLAGS)" docker-compose -f ./docker-compose/docker-compose-remote-postgres.yml up -d

# Stops and removes containers, networks, volumes, and images created by docker-compose up
down-postgres:
	docker-compose -f ./docker-compose/docker-compose-local-postgres.yml down

# Stops running containers without removing them. The containers can be restarted with make start
stop-postgres:
	docker-compose -f ./docker-compose/docker-compose-local-postgres.yml stop

# Starts existing containers that were stopped
start-postgres:
	docker-compose -f ./docker-compose/docker-compose-local-postgres.yml start

# Clean shared volume
clean-volume-postgres:
	@echo "Cleaning up postgres database volume"
	@sudo rm -rf postgres_data
	@sudo rm -rf ./docker-compose/postgres_data



build:
	$(CARGO) build --release

# Build the project in debug mode
debug:
	$(CARGO) build

# Run the project in release mode
run:
	$(CARGO) run --release

# Run the project in debug mode
run-debug:
	$(CARGO) run

# Test the project
test:
	$(CARGO) test --all-targets --all-features

# Clean the project
clean:
	$(CARGO) clean

# Format the code
fmt:
	$(CARGO) fmt

# Check for common mistakes
clippy:
	$(CARGO) clippy --all-targets --all-features -- -D warnings