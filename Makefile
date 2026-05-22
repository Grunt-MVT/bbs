SHELL := /usr/bin/env bash

ROOT_DIR := $(abspath $(dir $(lastword $(MAKEFILE_LIST))))
override CARGO_TARGET_DIR := $(ROOT_DIR)/target
TARGET_DIR := $(CARGO_TARGET_DIR)/release
DIST_DIR := $(ROOT_DIR)/dist
ARTIFACT_NAME := libbbsplus-linux-amd64
LINUX_SHARED_LIB := libbbsplus.so
DOCKER_IMAGE ?= libbbsplus-ci

.PHONY: test
test:
	CARGO_TARGET_DIR="$(CARGO_TARGET_DIR)" cargo test

.PHONY: build-release
build-release:
	CARGO_TARGET_DIR="$(CARGO_TARGET_DIR)" cargo build --release

.PHONY: test-go-ffi
test-go-ffi: build-release
	cd "$(ROOT_DIR)/tests/goffi" && \
		CGO_ENABLED=1 \
		CGO_CFLAGS="-I$(ROOT_DIR)/include" \
		CGO_LDFLAGS="-L$(TARGET_DIR) -lbbsplus" \
		LD_LIBRARY_PATH="$(TARGET_DIR):$${LD_LIBRARY_PATH:-}" \
		DYLD_LIBRARY_PATH="$(TARGET_DIR):$${DYLD_LIBRARY_PATH:-}" \
		go test -v ./...

.PHONY: package-linux-amd64
package-linux-amd64: build-release
	test -f "$(TARGET_DIR)/$(LINUX_SHARED_LIB)" || \
		(echo "missing $(TARGET_DIR)/$(LINUX_SHARED_LIB); run this target on Linux/amd64 or use make docker-artifacts" && exit 1)
	rm -rf "$(DIST_DIR)/$(ARTIFACT_NAME)" "$(DIST_DIR)/$(ARTIFACT_NAME).tar.gz"
	mkdir -p "$(DIST_DIR)/$(ARTIFACT_NAME)/lib" "$(DIST_DIR)/$(ARTIFACT_NAME)/include"
	cp "$(TARGET_DIR)/$(LINUX_SHARED_LIB)" "$(DIST_DIR)/$(ARTIFACT_NAME)/lib/"
	cp "$(ROOT_DIR)/include/bbs_ffi.h" "$(DIST_DIR)/$(ARTIFACT_NAME)/include/"
	tar -C "$(DIST_DIR)" -czf "$(DIST_DIR)/$(ARTIFACT_NAME).tar.gz" "$(ARTIFACT_NAME)"

.PHONY: ci
ci: test test-go-ffi package-linux-amd64

.PHONY: docker-ci
docker-ci:
	docker build --platform linux/amd64 --target ci -t "$(DOCKER_IMAGE)" "$(ROOT_DIR)"

.PHONY: docker-artifacts
docker-artifacts:
	rm -rf "$(DIST_DIR)"
	mkdir -p "$(DIST_DIR)"
	docker buildx build --platform linux/amd64 --target artifacts --output type=local,dest="$(DIST_DIR)" "$(ROOT_DIR)"
