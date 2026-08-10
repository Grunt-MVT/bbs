SHELL := /usr/bin/env bash

ROOT_DIR := $(abspath $(dir $(lastword $(MAKEFILE_LIST))))
override CARGO_TARGET_DIR := $(ROOT_DIR)/target
TARGET_DIR := $(CARGO_TARGET_DIR)/release
DIST_DIR := $(ROOT_DIR)/dist
LINUX_ARTIFACT_NAME := libbbsplus-linux-amd64
DARWIN_ARTIFACT_NAME := libbbsplus-darwin-arm64
STATIC_LIB := libbbsplus.a
GO_NATIVE_LINUX_AMD64_DIR := $(ROOT_DIR)/go/native/linux_amd64
GO_NATIVE_DARWIN_ARM64_DIR := $(ROOT_DIR)/go/native/darwin_arm64
NODE_NATIVE_LINUX_AMD64_DIR := $(ROOT_DIR)/node/native/linux_amd64
NODE_NATIVE_DARWIN_ARM64_DIR := $(ROOT_DIR)/node/native/darwin_arm64
DOCKER_IMAGE ?= libbbsplus-ci
DOCKER_LINUX_OUTPUT_DIR := $(DIST_DIR)/docker-linux-amd64

.PHONY: test
test:
	CARGO_TARGET_DIR="$(CARGO_TARGET_DIR)" cargo test

# Build the shared Rust/C core once per host target.
.PHONY: build-release
build-release:
	CARGO_TARGET_DIR="$(CARGO_TARGET_DIR)" cargo build --release

.PHONY: test-go-ffi
test-go-ffi:
	cd "$(ROOT_DIR)/go" && CGO_ENABLED=1 go test -v ./...

# Thin N-API (C++) adapter statically linked against the prebuilt libbbsplus.a.
.PHONY: build-node
build-node: build-release
	cd "$(ROOT_DIR)/node" && npm ci && \
		BBSPLUS_LIB_DIR="$(TARGET_DIR)" \
		npm run build

.PHONY: test-node
test-node: build-node
	cd "$(ROOT_DIR)/node" && npm test

.PHONY: sync-node-native-linux-amd64
sync-node-native-linux-amd64: build-node
	test "$$(uname -s)" = "Linux" && test "$$(uname -m)" = "x86_64" || \
		(echo "sync-node-native-linux-amd64 must run on Linux/amd64" && exit 1)
	test -f "$(NODE_NATIVE_LINUX_AMD64_DIR)/bbsplus_node.node" || \
		(echo "missing $(NODE_NATIVE_LINUX_AMD64_DIR)/bbsplus_node.node" && exit 1)

.PHONY: sync-node-native-darwin-arm64
sync-node-native-darwin-arm64: build-node
	test "$$(uname -s)" = "Darwin" && test "$$(uname -m)" = "arm64" || \
		(echo "sync-node-native-darwin-arm64 must run on Darwin/arm64" && exit 1)
	test -f "$(NODE_NATIVE_DARWIN_ARM64_DIR)/bbsplus_node.node" || \
		(echo "missing $(NODE_NATIVE_DARWIN_ARM64_DIR)/bbsplus_node.node" && exit 1)

.PHONY: sync-go-native-linux-amd64
sync-go-native-linux-amd64: build-release
	test "$$(uname -s)" = "Linux" && test "$$(uname -m)" = "x86_64" || \
		(echo "sync-go-native-linux-amd64 must run on Linux/amd64; use make docker-sync-linux-amd64" && exit 1)
	test -f "$(TARGET_DIR)/$(STATIC_LIB)" || \
		(echo "missing $(TARGET_DIR)/$(STATIC_LIB)" && exit 1)
	rm -rf "$(GO_NATIVE_LINUX_AMD64_DIR)"
	mkdir -p "$(GO_NATIVE_LINUX_AMD64_DIR)/lib" "$(GO_NATIVE_LINUX_AMD64_DIR)/include"
	cp "$(TARGET_DIR)/$(STATIC_LIB)" "$(GO_NATIVE_LINUX_AMD64_DIR)/lib/"
	cp "$(ROOT_DIR)/include/bbs_ffi.h" "$(GO_NATIVE_LINUX_AMD64_DIR)/include/"

.PHONY: sync-go-native-darwin-arm64
sync-go-native-darwin-arm64: build-release
	test "$$(uname -s)" = "Darwin" && test "$$(uname -m)" = "arm64" || \
		(echo "sync-go-native-darwin-arm64 must run on Darwin/arm64" && exit 1)
	test -f "$(TARGET_DIR)/$(STATIC_LIB)" || \
		(echo "missing $(TARGET_DIR)/$(STATIC_LIB)" && exit 1)
	rm -rf "$(GO_NATIVE_DARWIN_ARM64_DIR)"
	mkdir -p "$(GO_NATIVE_DARWIN_ARM64_DIR)/lib" "$(GO_NATIVE_DARWIN_ARM64_DIR)/include"
	cp "$(TARGET_DIR)/$(STATIC_LIB)" "$(GO_NATIVE_DARWIN_ARM64_DIR)/lib/"
	cp "$(ROOT_DIR)/include/bbs_ffi.h" "$(GO_NATIVE_DARWIN_ARM64_DIR)/include/"

.PHONY: package-linux-amd64
package-linux-amd64: build-release
	test "$$(uname -s)" = "Linux" && test "$$(uname -m)" = "x86_64" || \
		(echo "package-linux-amd64 must run on Linux/amd64; use make docker-sync-linux-amd64" && exit 1)
	test -f "$(TARGET_DIR)/$(STATIC_LIB)" || \
		(echo "missing $(TARGET_DIR)/$(STATIC_LIB); run this target on Linux/amd64 or use make docker-sync-linux-amd64" && exit 1)
	rm -rf "$(DIST_DIR)/$(LINUX_ARTIFACT_NAME)" "$(DIST_DIR)/$(LINUX_ARTIFACT_NAME).tar.gz"
	mkdir -p "$(DIST_DIR)/$(LINUX_ARTIFACT_NAME)/lib" "$(DIST_DIR)/$(LINUX_ARTIFACT_NAME)/include"
	cp "$(TARGET_DIR)/$(STATIC_LIB)" "$(DIST_DIR)/$(LINUX_ARTIFACT_NAME)/lib/"
	cp "$(ROOT_DIR)/include/bbs_ffi.h" "$(DIST_DIR)/$(LINUX_ARTIFACT_NAME)/include/"
	tar -C "$(DIST_DIR)" -czf "$(DIST_DIR)/$(LINUX_ARTIFACT_NAME).tar.gz" "$(LINUX_ARTIFACT_NAME)"

.PHONY: package-darwin-arm64
package-darwin-arm64: build-release
	test "$$(uname -s)" = "Darwin" && test "$$(uname -m)" = "arm64" || \
		(echo "package-darwin-arm64 must run on Darwin/arm64" && exit 1)
	test -f "$(TARGET_DIR)/$(STATIC_LIB)" || \
		(echo "missing $(TARGET_DIR)/$(STATIC_LIB)" && exit 1)
	rm -rf "$(DIST_DIR)/$(DARWIN_ARTIFACT_NAME)" "$(DIST_DIR)/$(DARWIN_ARTIFACT_NAME).tar.gz"
	mkdir -p "$(DIST_DIR)/$(DARWIN_ARTIFACT_NAME)/lib" "$(DIST_DIR)/$(DARWIN_ARTIFACT_NAME)/include"
	cp "$(TARGET_DIR)/$(STATIC_LIB)" "$(DIST_DIR)/$(DARWIN_ARTIFACT_NAME)/lib/"
	cp "$(ROOT_DIR)/include/bbs_ffi.h" "$(DIST_DIR)/$(DARWIN_ARTIFACT_NAME)/include/"
	tar -C "$(DIST_DIR)" -czf "$(DIST_DIR)/$(DARWIN_ARTIFACT_NAME).tar.gz" "$(DARWIN_ARTIFACT_NAME)"

# Core once, then Go + Node bindings against the same archive; test Node without rebuilding.
.PHONY: ci
ci: test build-release sync-go-native-linux-amd64 test-go-ffi sync-node-native-linux-amd64
	cd "$(ROOT_DIR)/node" && npm test
	$(MAKE) package-linux-amd64

.PHONY: docker-ci
docker-ci:
	docker build --platform linux/amd64 --target ci -t "$(DOCKER_IMAGE)" "$(ROOT_DIR)"

# One Docker build: shared core, Go native, Node native, and release tarball.
.PHONY: docker-sync-linux-amd64
docker-sync-linux-amd64:
	rm -rf "$(DOCKER_LINUX_OUTPUT_DIR)"
	mkdir -p "$(DOCKER_LINUX_OUTPUT_DIR)" "$(DIST_DIR)"
	docker buildx build --platform linux/amd64 --target linux-outputs \
		--output type=local,dest="$(DOCKER_LINUX_OUTPUT_DIR)" "$(ROOT_DIR)"
	test -d "$(DOCKER_LINUX_OUTPUT_DIR)/go-native" || \
		(echo "missing Docker go-native export" && exit 1)
	test -f "$(DOCKER_LINUX_OUTPUT_DIR)/node-native/bbsplus_node.node" || \
		(echo "missing Docker node-native export" && exit 1)
	test -f "$(DOCKER_LINUX_OUTPUT_DIR)/libbbsplus-linux-amd64.tar.gz" || \
		(echo "missing Docker release tarball export" && exit 1)
	rm -rf "$(GO_NATIVE_LINUX_AMD64_DIR)" "$(NODE_NATIVE_LINUX_AMD64_DIR)"
	mkdir -p "$(GO_NATIVE_LINUX_AMD64_DIR)" "$(NODE_NATIVE_LINUX_AMD64_DIR)"
	cp -R "$(DOCKER_LINUX_OUTPUT_DIR)/go-native/." "$(GO_NATIVE_LINUX_AMD64_DIR)/"
	cp -R "$(DOCKER_LINUX_OUTPUT_DIR)/node-native/." "$(NODE_NATIVE_LINUX_AMD64_DIR)/"
	cp "$(DOCKER_LINUX_OUTPUT_DIR)/libbbsplus-linux-amd64.tar.gz" "$(DIST_DIR)/$(LINUX_ARTIFACT_NAME).tar.gz"

# Backward-compatible aliases: both reuse the unified Docker export (no second compile).
.PHONY: docker-artifacts
docker-artifacts: docker-sync-linux-amd64

.PHONY: docker-sync-go-native-linux-amd64
docker-sync-go-native-linux-amd64: docker-sync-linux-amd64
