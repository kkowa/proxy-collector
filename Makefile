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

LOCBIN := ${PWD}/.bin
PATH := ${PATH}:${LOCBIN}


# =============================================================================
# Common
# =============================================================================
OPENAPI_GENERATOR_CLI_VERSION := $(shell sed -nE 's/ARG OPENAPI_GENERATOR_CLI_VERSION=\"(.+)\"/\1/p' Dockerfile)

install:  ## Install the app locally
	mkdir -p "${LOCBIN}"
	! command -v openapi-generator-cli > /dev/null \
		&& curl -fsSL -o "${LOCBIN}/openapi-generator-cli.jar" "https://repo1.maven.org/maven2/org/openapitools/openapi-generator-cli/${OPENAPI_GENERATOR_CLI_VERSION}/openapi-generator-cli-${OPENAPI_GENERATOR_CLI_VERSION}.jar" \
		&& chmod +x "${LOCBIN}/openapi-generator-cli.jar"

	cargo install cargo-watch grcov
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

generate:  ## Generate codes from schemas
	java -jar "$$(which openapi-generator-cli.jar)" generate \
		--input-spec idl/openapi/schemas/server.openapi.json \
		--output _generated/openapi/server \
		--generator-name rust \
		--package-name server-openapi \
		--library reqwest
.PHONY: generate

format:  ## Run autoformatters
	cargo fmt
	cargo clippy --fix --allow-dirty --allow-staged --allow-no-vcs
.PHONY: format

lint:  ## Run all linters
	cargo fmt --check
	cargo clippy
.PHONY: lint

# https://doc.rust-lang.org/rustc/instrument-coverage.html
# https://github.com/mozilla/grcov
test:  ## Run tests
	mkdir -p .report
	RUSTFLAGS='-C instrument-coverage' LLVM_PROFILE_FILE='.profile/proxy-%p-%m.profraw' \
		cargo test --workspace --target-dir target/.coverage -- -Z unstable-options --format junit --report-time > .report/raw

	split -l1 -d --additional-suffix=.xml .report/raw .report/partial.

	echo 'Generating coverage report in HTML format'
	grcov . \
		--llvm \
		--branch \
		--source-dir . \
		--ignore-not-existing \
		--ignore 'target/*' \
		--ignore '_generated/*' \
		--binary-path target/.coverage/debug/ \
		--output-type html \
		--output-path .coverage/html/

	echo 'Generating coverage report in cobertura XML format'
	grcov . \
		--llvm \
		--branch \
		--source-dir . \
		--ignore-not-existing \
		--ignore 'target/*' \
		--ignore '_generated/*' \
		--binary-path target/.coverage/debug/ \
		--output-type cobertura \
		--output-path .coverage/coverage.xml
.PHONY: test

scan:  ## Run all scans

.PHONY: scan


# =============================================================================
# Handy Scripts
# =============================================================================
clean:  ## Remove temporary files
	rm --recursive --force .coverage/ .profile/ .report/
	find . -path '*.log*' -delete
.PHONY: clean
