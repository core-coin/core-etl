# Variables
CORE_ETL_FLAGS ?= -s /data/data.sqld/dbs/default/data
CORE_ETL_EXPORT_FLAGS ?= -w="ctn" -l
GO_CORE_FLAGS ?= --ws --ws.addr 0.0.0.0 --syncmode fast --cache=128 --snapshot=false

# Targets
.PHONY: build up down clean 

clean:
	@echo "Cleaning up core-etl.log, sqlite3.db and db.db"
	@rm -f core-etl.log
	@rm -f sqlite3.db sqlite3.db-shm sqlite3.db-wal sqlite3_dump.sql
	@rm -f db.db db.db-shm db.db-wal db_dump.sql

# Build Docker images
build:
	docker-compose -f docker-compose-local.yml build

# Initialize database mount that is used both by core-etl and libsql-server
init:
	@echo "Initializing database mount"
	docker-compose -f docker-compose-init.yml up  -d
	@sleep 3
	docker-compose -f docker-compose-init.yml down
# Builds, (re)creates, starts, and attaches to containers for a service
# This runs the additional go-core container 
sync-local:
	GO_CORE_FLAGS="$(GO_CORE_FLAGS)" CORE_ETL_FLAGS="-r ws://go-core:8546 $(CORE_ETL_FLAGS)" CORE_ETL_EXPORT_FLAGS="$(CORE_ETL_EXPORT_FLAGS)" docker-compose -f docker-compose-local.yml up -d

# Builds, (re)creates, starts, and attaches to containers for a service
# This runs the additional go-core container 
sync-remote:
	CORE_ETL_FLAGS="-n mainnet $(CORE_ETL_FLAGS)" CORE_ETL_EXPORT_FLAGS="$(CORE_ETL_EXPORT_FLAGS)" docker-compose -f docker-compose-remote.yml up -d

# Stops and removes containers, networks, volumes, and images created by docker-compose up
down:
	docker-compose -f docker-compose-local.yml down

# Stops running containers without removing them. The containers can be restarted with make start
stop:
	docker-compose -f docker-compose-local.yml stop

# Starts existing containers that were stopped
start:
	docker-compose -f docker-compose-local.yml start

# Clean shared volume
clean-volume:
	@echo "Cleaning up libsql database volume"
	@sudo rm -rf data
