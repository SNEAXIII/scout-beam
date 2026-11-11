# Recipes are POSIX: on Windows, point make at the bash Git ships (override: make GIT_BASH=<path>).
# The probe uses the 8.3 path because make's $(wildcard) splits on spaces.
ifeq ($(OS),Windows_NT)
GIT_BASH ?= C:/Program Files/Git/bin/bash.exe
ifneq ($(wildcard C:/PROGRA~1/Git/bin/bash.exe),)
SHELL := $(GIT_BASH)
.SHELLFLAGS := -c
else
$(warning Git Bash not found at $(GIT_BASH) -- pass GIT_BASH=<path to bash.exe>)
endif
endif

# No ANSI colours: Windows PowerShell 5.1 prints the escape codes verbatim.

.PHONY: sniff help tools db db-stop db-reset db-fresh print-db-name print-db-url print-db-container print-test-db-url migrate migrate-info prepare sqlx-check image dev kill seed-user test coverage coverage-html lint fmt check ci
.DEFAULT_GOAL := help

DB_CONTAINER := postgres-scout-beam
PORT := 8002
LOGIN ?=
ROLE ?= user
COMPOSE := docker compose -f compose.yaml

# Throwaway instance for `make test` only (no fsync, tmpfs); overridable so CI can bring its own.
TEST_CONTAINER := postgres-scout-beam-test
TEST_DB := scout_beam_test
TEST_DB_URL ?= postgres://user:password@localhost:5437/$(TEST_DB)

# The database `.env` points at: never dropped by a make target.
DEV_DB := scout_beam
# The scratch database `db-fresh` rebuilds, never defaulted to DEV_DB.
DB_NAME ?= scout_beam_migrate
DB_URL ?= postgres://user:password@localhost:5436/$(DB_NAME)

help: ## Show this help
	@grep -E '^[a-z-]+:.*?## .*$$' $(MAKEFILE_LIST) | awk 'BEGIN {FS = ":.*?## "}; {printf "  %-12s %s\n", $$1, $$2}'

tools: ## Install the dev tooling (sqlx-cli, cargo-llvm-cov, cargo-nextest)
	cargo install sqlx-cli --no-default-features --features rustls,postgres
	rustup component add llvm-tools-preview
	cargo install cargo-llvm-cov
	cargo install cargo-nextest --locked

db: ## Start both PostgreSQL instances (dev 5436, test 5437)
	$(COMPOSE) up -d
	@echo "waiting for postgres..."
	@until docker exec $(DB_CONTAINER) pg_isready -U user -d $(DEV_DB) >/dev/null 2>&1; do sleep 1; done
	@until docker exec $(TEST_CONTAINER) pg_isready -U user -d $(TEST_DB) >/dev/null 2>&1; do sleep 1; done
	@echo "postgres is ready (dev 5436, test 5437)"

db-stop: ## Stop PostgreSQL, keep the data
	$(COMPOSE) down

db-reset: db-stop ## Drop the WHOLE instance (volume included) and migrate from scratch
	$(COMPOSE) down -v
	$(MAKE) db
	$(MAKE) migrate

db-fresh: ## Rebuild ONE database from the migrations: make db-fresh [DB_NAME=scout_beam_migrate]
	@command -v sqlx >/dev/null 2>&1 || { echo "sqlx-cli is missing -- run: make tools"; exit 1; }
	@test "$(DB_NAME)" != "$(DEV_DB)" || { echo "refusing to rebuild the dev database ($(DEV_DB)) -- use a scratch DB_NAME, or make db-reset on purpose"; exit 1; }
	sqlx database drop -y -D "$(DB_URL)"
	sqlx database create -D "$(DB_URL)"
	sqlx migrate run -D "$(DB_URL)"
	@echo "$(DB_NAME) rebuilt -- validate the macros against it with:"
	@echo "  DATABASE_URL=$(DB_URL) cargo build"

print-db-name: ## Print the scratch database name
	@echo $(DB_NAME)

print-db-url: ## Print the scratch database URL
	@echo $(DB_URL)

print-db-container: ## Print the PostgreSQL container name
	@echo $(DB_CONTAINER)

print-test-db-url: ## Print the throwaway test instance URL
	@echo $(TEST_DB_URL)

migrate: ## Apply pending migrations (idempotent: sqlx tracks what ran)
	@command -v sqlx >/dev/null 2>&1 || { echo "sqlx-cli is missing -- run: make tools"; exit 1; }
	sqlx migrate run

migrate-info: ## Show which migrations are applied
	sqlx migrate info

# The image build has no database: run after touching a query or migration, and commit `.sqlx/`.
prepare: ## Refresh the .sqlx offline cache (needed by the Docker build)
	@command -v sqlx >/dev/null 2>&1 || { echo "sqlx-cli is missing -- run: make tools"; exit 1; }
	cargo sqlx prepare

sqlx-check: ## Fail if the .sqlx cache no longer matches the schema
	@command -v sqlx >/dev/null 2>&1 || { echo "sqlx-cli is missing -- run: make tools"; exit 1; }
	@cargo sqlx prepare --check || { echo "stale .sqlx cache -- run: make prepare && git add .sqlx"; exit 1; }

image: ## Build the Docker image locally (no database needed, uses .sqlx)
	docker build -t scout-beam:local .

dev: ## Run the API (http://localhost:8002)
	cargo run

kill: ## Kill whatever still listens on the API port (see scripts/kill-port.ps1)
	@powershell -NoProfile -ExecutionPolicy Bypass -File scripts/kill-port.ps1 -Port $(PORT)

seed-user: ## Create or promote an account: LOGIN=x [ROLE=admin|super_admin]
	@test -n "$(LOGIN)" || (echo "usage: make seed-user LOGIN=Sneaxiii [ROLE=admin]"; exit 1)
	cargo run --quiet --bin seed_user -- --login "$(LOGIN)" --role "$(ROLE)"

# The test instance is a tmpfs, empty on every `up`: migrate first, the query! macros need the schema.
test: ## Run unit and integration tests (throwaway instance, port 5437)
	@command -v sqlx >/dev/null 2>&1 || { echo "sqlx-cli is missing -- run: make tools"; exit 1; }
	@sqlx migrate run -D "$(TEST_DB_URL)" >/dev/null
	@if command -v cargo-nextest >/dev/null 2>&1; then \
		DATABASE_URL="$(TEST_DB_URL)" cargo nextest run; \
	else \
		echo "cargo-nextest not installed (make tools) -- falling back to cargo test"; \
		DATABASE_URL="$(TEST_DB_URL)" cargo test; \
	fi

coverage: ## Per-file coverage table (throwaway instance)
	@command -v cargo-llvm-cov >/dev/null 2>&1 || { echo "cargo-llvm-cov is missing -- run: make tools"; exit 1; }
	@sqlx migrate run -D "$(TEST_DB_URL)" >/dev/null
	DATABASE_URL="$(TEST_DB_URL)" cargo llvm-cov --workspace

coverage-html: ## Same, as a browsable HTML report
	@sqlx migrate run -D "$(TEST_DB_URL)" >/dev/null
	DATABASE_URL="$(TEST_DB_URL)" cargo llvm-cov --workspace --html --open

lint: ## Clippy, warnings are errors
	cargo clippy --all-targets -- -D warnings

# Probe by running it: on Windows `python3` is the Store stub, on Linux `python` may not exist.
PYTHON ?= $(shell python3 -c "" >/dev/null 2>&1 && echo python3 || echo python)

sniff: ## Project-specific smells clippy cannot see (duplicated bounds, bare #[allow], untransacted writes)
	@$(PYTHON) scripts/sniff.py

fmt: ## Format the workspace
	cargo fmt

check: fmt lint sqlx-check test ## Format, lint, cache and tests -- run before pushing

ci: lint test ## What the pipeline runs
