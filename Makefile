# PM Endgame Sweep - Makefile
# Run `make help` for available commands

.PHONY: help
help: ## Show this help message
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-25s\033[0m %s\n", $$1, $$2}'

# =============================================================================
# Configuration
# =============================================================================

RUST_LOG ?= info
DATABASE_URL ?= postgres://postgres:postgres@localhost:5432/pm_endgame
WEB_PORT ?= 3001
API_PORT ?= 8080

export RUST_LOG
export DATABASE_URL

# =============================================================================
# Development Setup
# =============================================================================

.PHONY: setup
setup: install-tools install-deps db-setup ## Complete development setup

.PHONY: install-tools
install-tools: ## Install required development tools
	@echo "Installing Rust tools..."
	cargo install sqlx-cli --no-default-features --features postgres
	cargo install cargo-watch
	cargo install cargo-audit
	cargo install cargo-deny
	@echo "Installing Playwright browsers..."
	cd web && yarn playwright install

.PHONY: install-deps
install-deps: ## Install all dependencies
	@echo "Installing Rust dependencies..."
	cargo fetch
	@echo "Installing web dependencies..."
	cd web && yarn install

# =============================================================================
# Database
# =============================================================================

.PHONY: db-up
db-up: ## Start PostgreSQL container
	docker compose up -d postgres
	@echo "Waiting for PostgreSQL to be ready..."
	@sleep 3

.PHONY: db-down
db-down: ## Stop PostgreSQL container
	docker compose down

.PHONY: db-setup
db-setup: db-up db-migrate ## Start database and run migrations

.PHONY: db-migrate
db-migrate: ## Run database migrations
	sqlx migrate run --source migrations

.PHONY: db-reset
db-reset: ## Reset database (drop and recreate)
	sqlx database drop -y || true
	sqlx database create
	sqlx migrate run --source migrations

.PHONY: db-shell
db-shell: ## Open PostgreSQL shell
	docker compose exec postgres psql -U postgres -d pm_endgame

# =============================================================================
# Backend (Rust)
# =============================================================================

.PHONY: build
build: ## Build all Rust crates (release)
	cargo build --release

.PHONY: build-debug
build-debug: ## Build all Rust crates (debug)
	cargo build

.PHONY: check
check: ## Check Rust code compiles
	cargo check --all-targets --all-features

.PHONY: test
test: ## Run Rust tests
	cargo test --all-features

.PHONY: test-verbose
test-verbose: ## Run Rust tests with verbose output
	cargo test --all-features -- --nocapture

.PHONY: fmt
fmt: ## Format Rust code
	cargo fmt --all

.PHONY: fmt-check
fmt-check: ## Check Rust code formatting
	cargo fmt --all -- --check

.PHONY: lint
lint: ## Run Clippy linter
	cargo clippy --all-targets --all-features -- -D warnings

.PHONY: audit
audit: ## Run security audit on Rust dependencies
	cargo audit --deny warnings

.PHONY: deny
deny: ## Run cargo-deny license and advisory checks
	cargo deny check

.PHONY: backend-ci
backend-ci: fmt-check lint test audit deny ## Run all backend CI checks

# =============================================================================
# Frontend (Next.js)
# =============================================================================

.PHONY: web-install
web-install: ## Install web dependencies
	cd web && yarn install

.PHONY: web-build
web-build: ## Build web application
	cd web && yarn build

.PHONY: web-dev
web-dev: ## Start web development server
	cd web && yarn dev

.PHONY: web-start
web-start: ## Start web production server
	cd web && yarn start

.PHONY: web-lint
web-lint: ## Lint web code
	cd web && yarn lint

.PHONY: web-test
web-test: ## Run web unit tests
	cd web && yarn test

.PHONY: web-test-watch
web-test-watch: ## Run web tests in watch mode
	cd web && yarn test --watch

.PHONY: web-test-coverage
web-test-coverage: ## Run web tests with coverage
	cd web && yarn test:coverage

.PHONY: web-test-ui
web-test-ui: ## Run web tests with UI
	cd web && yarn test:ui

.PHONY: web-e2e
web-e2e: ## Run Playwright e2e tests
	cd web && yarn test:e2e

.PHONY: web-e2e-chromium
web-e2e-chromium: ## Run Playwright e2e tests (Chromium only)
	cd web && yarn test:e2e --project=chromium

.PHONY: web-e2e-ui
web-e2e-ui: ## Run Playwright e2e tests with UI
	cd web && yarn test:e2e:ui

.PHONY: web-e2e-debug
web-e2e-debug: ## Run Playwright e2e tests in debug mode
	cd web && yarn test:e2e:debug

.PHONY: web-ci
web-ci: web-lint web-test web-build ## Run all web CI checks

# =============================================================================
# Full Stack
# =============================================================================

.PHONY: dev
dev: ## Start all services for development (requires tmux or multiple terminals)
	@echo "Starting development servers..."
	@echo "Run these in separate terminals:"
	@echo "  make db-up"
	@echo "  make run-api"
	@echo "  make run-ingest"
	@echo "  make run-scoring"
	@echo "  make web-dev"

.PHONY: run-api
run-api: ## Run API service
	cargo run --bin pm-api

.PHONY: run-ingest
run-ingest: ## Run ingest service
	cargo run --bin pm-ingest

.PHONY: run-scoring
run-scoring: ## Run scoring service
	cargo run --bin pm-scoring

.PHONY: watch-api
watch-api: ## Run API service with auto-reload
	cargo watch -x 'run --bin pm-api'

.PHONY: watch-ingest
watch-ingest: ## Run ingest service with auto-reload
	cargo watch -x 'run --bin pm-ingest'

.PHONY: watch-scoring
watch-scoring: ## Run scoring service with auto-reload
	cargo watch -x 'run --bin pm-scoring'

# =============================================================================
# Testing
# =============================================================================

.PHONY: test-all
test-all: test web-test ## Run all tests (backend + frontend)

.PHONY: test-integration
test-integration: db-setup ## Run integration tests (requires database)
	cargo test --all-features -- --ignored

.PHONY: test-e2e
test-e2e: web-e2e ## Run end-to-end tests

.PHONY: coverage
coverage: ## Generate test coverage report
	@echo "Backend coverage..."
	cargo tarpaulin --out Html --output-dir target/coverage || echo "Install: cargo install cargo-tarpaulin"
	@echo "Frontend coverage..."
	cd web && yarn test:coverage

# =============================================================================
# Security
# =============================================================================

.PHONY: security
security: audit deny ## Run all security checks

.PHONY: security-full
security-full: audit deny ## Run comprehensive security scan
	@echo "Running security checks..."
	cargo audit --deny warnings
	cargo deny check
	@echo "Security checks complete."

# =============================================================================
# CI/CD
# =============================================================================

.PHONY: ci
ci: backend-ci web-ci ## Run all CI checks

.PHONY: ci-quick
ci-quick: check fmt-check lint web-lint ## Quick CI checks (no tests)

# =============================================================================
# Cleanup
# =============================================================================

.PHONY: clean
clean: ## Clean build artifacts
	cargo clean
	rm -rf web/.next
	rm -rf web/node_modules/.cache

.PHONY: clean-all
clean-all: clean ## Clean everything including dependencies
	rm -rf target
	rm -rf web/node_modules
	rm -rf web/coverage
	rm -rf web/playwright-report
	rm -rf web/test-results

.PHONY: reset
reset: clean-all db-reset setup ## Full reset and reinstall

# =============================================================================
# Docker
# =============================================================================

.PHONY: docker-up
docker-up: ## Start all services with Docker Compose
	docker compose up

.PHONY: docker-up-d
docker-up-d: ## Start all services with Docker Compose (detached)
	docker compose up -d

.PHONY: docker-down
docker-down: ## Stop all Docker services
	docker compose down

.PHONY: docker-build
docker-build: ## Build Docker images
	docker compose build

.PHONY: docker-logs
docker-logs: ## Show Docker logs
	docker compose logs -f

# =============================================================================
# Documentation
# =============================================================================

.PHONY: docs
docs: ## Generate Rust documentation
	cargo doc --no-deps --open

.PHONY: docs-build
docs-build: ## Build Rust documentation
	cargo doc --no-deps

# =============================================================================
# Utility
# =============================================================================

.PHONY: loc
loc: ## Count lines of code
	@echo "Rust:"
	@find crates -name '*.rs' | xargs wc -l | tail -1
	@echo "TypeScript/JavaScript:"
	@find web -name '*.ts' -o -name '*.tsx' | grep -v node_modules | grep -v .next | xargs wc -l | tail -1

.PHONY: todo
todo: ## Find TODO/FIXME comments
	@echo "Backend TODOs:"
	@grep -rn "TODO\|FIXME" crates --include="*.rs" || true
	@echo ""
	@echo "Frontend TODOs:"
	@grep -rn "TODO\|FIXME" web --include="*.ts" --include="*.tsx" | grep -v node_modules || true

.PHONY: health
health: ## Check service health
	@echo "API Health:"
	@curl -s http://localhost:$(API_PORT)/healthz || echo "API not running"
	@echo ""
	@echo "Database:"
	@docker compose exec -T postgres pg_isready -U postgres || echo "Database not running"

