.PHONY: run run-docker seed check test build

# Default: Run locally
run:
	@./scripts/run_local.sh

# Run with Docker Compose
run-docker:
	docker compose up --build

# Seed the knowledge base
seed:
	@./scripts/seed_knowledge.sh

# Check code without building
check:
	cargo check

# Run tests
test:
	cargo test

# Build release binary
build:
	cargo build --release
