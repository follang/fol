PROJECT_NAME := $(shell grep '^name = ' Cargo.toml | sed -E 's/name = "(.*)"/\1/')
PROJECT_CAP  := $(shell echo $(PROJECT_NAME) | tr '[:lower:]' '[:upper:]')
CURRENT_VERSION := $(shell grep '^version = ' Cargo.toml | sed -E 's/version = "(.*)"/\1/')
LATEST_TAG   ?= $(shell git describe --tags --abbrev=0 2>/dev/null)
TOP_DIR      := $(CURDIR)
BUILD_DIR    := $(TOP_DIR)/target

ifeq ($(PROJECT_NAME),)
$(error Error: project name not found in Cargo.toml)
endif

$(info ------------------------------------------)
$(info Project: $(PROJECT_NAME))
$(info Version: $(CURRENT_VERSION))
$(info ------------------------------------------)

.PHONY: build b compile c run r test t help h clean docs release

SHELL := /bin/bash


build:
	@cargo build --release

b: build

compile:
	@cargo clean
	@make build

c: compile

run:
	@cargo run

r: run

# Run the built binary directly (like run.sh does)
run-binary:
	@./target/release/fol

rb: run-binary

test:
	@cargo test
	@cargo test -- --ignored

t: test

help:
	@echo
	@echo "Usage: make [target]"
	@echo
	@echo "Available targets:"
	@echo "  build        Build project"
	@echo "  compile      Configure and generate build files"
	@echo "  run          Run the main executable"
	@echo "  test         Run tests"
	@echo "  docs         Build documentation (TYPE=mdbook|doxygen)"
	@echo "  release      Create a new release (TYPE=patch|minor|major)"
	@echo

h : help

clean:
	@echo "Cleaning build directory..."
	@rm -rf $(BUILD_DIR)
	@echo "Build directory cleaned."

docs:
ifeq ($(TYPE),mdbook)
	@command -v mdbook >/dev/null 2>&1 || { echo "mdbook is not installed. Please install it first."; exit 1; }
	@mdbook build $(TOP_DIR)/book --dest-dir $(TOP_DIR)/docs
	@git add --all && git commit -m "docs: building website/mdbook"
else ifeq ($(TYPE),doxygen)
	@command -v doxygen >/dev/null 2>&1 || { echo "doxygen is not installed. Please install it first."; exit 1; }
else
	$(error Invalid documentation type. Use 'make docs TYPE=mdbook' or 'make docs TYPE=doxygen')
endif


release:
	@if [ -z "$(TYPE)" ]; then \
		echo "Release type not specified. Use 'make release TYPE=[patch|minor|major]'"; \
		exit 1; \
	fi; \
	CURRENT_VERSION=$$(grep '^version = ' Cargo.toml | sed -E 's/version = "(.*)"/\1/'); \
	IFS='.' read -r MAJOR MINOR PATCH <<< "$$CURRENT_VERSION"; \
	case "$(TYPE)" in \
		major) MAJOR=$$((MAJOR+1)); MINOR=0; PATCH=0 ;; \
		minor) MINOR=$$((MINOR+1)); PATCH=0 ;; \
		patch) PATCH=$$((PATCH+1)); ;; \
		*) echo "Invalid release type. Use patch, minor or major."; exit 1 ;; \
	esac; \
	version="$$MAJOR.$$MINOR.$$PATCH"; \
	if [ -n "$(LATEST_TAG)" ]; then \
		changelog=$$(git cliff $(LATEST_TAG)..HEAD --strip all); \
		git cliff --tag $$version $(LATEST_TAG)..HEAD --prepend CHANGELOG.md; \
	else \
		changelog=$$(git cliff --unreleased --strip all); \
		git cliff --tag $$version --unreleased --prepend CHANGELOG.md; \
	fi; \
	sed -i 's/^version = ".*"/version = "'$$version'"/' Cargo.toml; \
	git add -A && git commit -m "chore(release): prepare for $$version"; \
	echo "$$changelog"; \
	git tag -a $$version -m "$$version" -m "$$changelog"; \
	git push --follow-tags --force --set-upstream origin develop; \
	gh release create $$version --notes "$$changelog"
