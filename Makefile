# Variables
ENDPOINT_URL ?= ws://127.0.0.1:8546
SQLITE3_DB_FILE ?= sqlite3.db
FLAGS ?=
D1_DATABASE ?= default-d1-database
CLOUDFLARE_API_TOKEN ?= 

export CLOUDFLARE_API_TOKEN

# Targets
.PHONY: initial_d1 run_etl_sqlite dump_db clean_sql_for_d1 execute_sql run_etl_d1 clean_sqlite build continue_d1

initial_d1: run_etl_sqlite dump_db clean_sql_for_d1 execute_sql run_etl_d1
continue_d1: run_etl_d1

run_etl_sqlite:
	@echo "Running core-etl with endpoint: $(ENDPOINT_URL), database: $(SQLITE3_DB_FILE), flags: $(FLAGS)"
	@bash -c ' \
	set -e; \
	touch core-etl.log; \
	./target/debug/core-etl -r $(ENDPOINT_URL) -s $(SQLITE3_DB_FILE) export $(FLAGS) 2>&1 | tee core-etl.log & \
	PID=$$!; \
	echo "core-etl PID: $$PID"; \
	trap "echo Stopping core-etl; kill -TERM $$PID; wait $$PID" SIGINT SIGTERM; \
	while kill -0 $$PID 2>/dev/null; do \
		if grep -q "Imported new block" core-etl.log; then \
			echo "Log detected: Imported new block"; \
			kill -TERM $$PID; \
			break; \
		fi; \
		sleep 1; \
	done; \
	EXIT_CODE=$$?; \
	if [ $$EXIT_CODE -eq 143 ]; then \
		echo "core-etl was terminated by SIGTERM"; \
	elif [ $$EXIT_CODE -eq 0 ]; then \
		echo "core-etl completed successfully"; \
	else \
		echo "core-etl exited with code $$EXIT_CODE"; \
	fi; \
	exit $$EXIT_CODE; \
	'
	
dump_db:
	@echo "Dumping SQLite database to sqlite3_dump.sql"
	@sqlite3 $(SQLITE3_DB_FILE) .dump > sqlite3_dump.sql

clean_sql_for_d1:
	@echo "Cleaning sqlite3_dump.sql"
	@sed -i '/PRAGMA foreign_keys=OFF;/d' sqlite3_dump.sql
	@sed -i '/BEGIN TRANSACTION;/d' sqlite3_dump.sql
	@sed -i '/COMMIT;/d' sqlite3_dump.sql

execute_sql:
	@echo "Executing sqlite3_dump.sql on D1 database"
	@npx wrangler d1 execute $(D1_DATABASE) -y --remote --file=sqlite3_dump.sql

run_etl_d1:
	@echo "Running core-etl with endpoint: $(ENDPOINT_URL), D1 database: $(D1_DATABASE), flags: $(FLAGS)"
	@bash -c ' \
	set -e; \
	./target/debug/core-etl -r $(ENDPOINT_URL) -d $(D1_DATABASE) --storage d1-storage export $(FLAGS) & \
	PID=$$!; \
	echo "core-etl PID: $$PID"; \
	trap "echo Stopping core-etl; kill -TERM $$PID; wait $$PID" SIGINT SIGTERM; \
	wait $$PID; \
	EXIT_CODE=$$?; \
	if [ $$EXIT_CODE -eq 143 ]; then \
		echo "core-etl was terminated by SIGTERM"; \
	elif [ $$EXIT_CODE -eq 0 ]; then \
		echo "core-etl completed successfully"; \
	else \
		echo "core-etl exited with code $$EXIT_CODE"; \
	fi; \
	exit $$EXIT_CODE; \
	'

clean:
	@echo "Cleaning up core-etl.log, sqlite3.db and db.db"
	@rm -f core-etl.log
	@rm -f sqlite3.db sqlite3.db-shm sqlite3.db-wal sqlite3_dump.sql
	@rm -f db.db db.db-shm db.db-wal db_dump.sql

build:
	@echo "Building core-etl"
	@cargo build
