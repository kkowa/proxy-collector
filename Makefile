#!/usr/bin/env make -f

MAKEFLAGS += --warn-undefined-variables
MAKEFLAGS += --no-builtin-rules
MAKEFLAGS += --silent

SHELL := bash
.ONESHELL:
.SHELLFLAGS := -eu -o pipefail -c
.DELETE_ON_ERROR:
.DEFAULT_GOAL := help
help: Makefile
	@grep -E '(^[a-zA-Z_-]+:.*?##.*$$)|(^##)' $(MAKEFILE_LIST) | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[32m%-30s\033[0m %s\n", $$1, $$2}' | sed -e 's/\[32m##/[33m/'


# =============================================================================
# Common
# =============================================================================
install:  ## Install the app locally
	cargo fetch
.PHONY: install

init:  ## Initialize project repository
	git submodule update --init
	pre-commit autoupdate
	pre-commit install --install-hooks --hook-type pre-commit --hook-type commit-msg
.PHONY: init

run:  ## Run development server
	cargo watch --no-gitignore --why --exec "run -- --verbosity debug"
.PHONY: run


# =============================================================================
# CI
# =============================================================================
ci: lint test scan  ## Run CI tasks
.PHONY: ci

format:  ## Run autoformatters
	cargo fmt
	cargo clippy --fix --allow-dirty --allow-staged --allow-no-vcs
.PHONY: format

lint:  ## Run all linters
	cargo fmt --check
	cargo clippy
.PHONY: lint

test:  ## Run tests
	raw="$$(mktemp)"
	reports="$${PWD}/.reports"
	RUSTFLAGS='-C instrument-coverage' LLVM_PROFILE_FILE='.profile/proxy-%m.profraw' cargo test -- -Z unstable-options --format junit --report-time > $${raw}
	mkdir -p $${reports}
	split -l1 -d --additional-suffix='.xml' $${raw} "$${reports}/partial."
	grcov --llvm --branch --ignore-not-existing --source-dir . --keep-only 'src/**/*.rs' --binary-path target/debug/ --output-type html --output-path .coverage/ .
	grcov --llvm --branch --ignore-not-existing --source-dir . --keep-only 'src/**/*.rs' --binary-path target/debug/ --output-type cobertura --output-path coverage.xml .
.PHONY: test

scan:  ## Run all scans

.PHONY: scan


# =============================================================================
# Handy Scripts
# =============================================================================
clean:  ## Remove temporary files
	rm --recursive --force .coverage/ .profile/ .reports/ coverage.xml
	find . -path '*.log*' -delete
.PHONY: clean
