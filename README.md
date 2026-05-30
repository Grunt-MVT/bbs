# libbbsplus

`libbbsplus` is a small FFI wrapper over Dock's BBS+ Rust library. It exposes a C ABI and a Go package so services written in Go can generate keys, sign and verify messages, and create and verify selective-disclosure proofs without binding directly to Dock's Rust types.

The Go package vendors static native archives for Linux AMD64 and Apple Silicon macOS, so users on those targets do not need to pass cgo compiler/linker flags or configure runtime library paths. Public native functions keep the `bbs_` prefix and are declared in [`include/bbs_ffi.h`](include/bbs_ffi.h).

## What It Exposes

- `bbs_generate_keypair` creates Dock-compatible signature parameters, secret key, and public key bytes.
- `bbs_sign` signs raw byte messages. Messages are hashed to BLS12-381 field elements inside the library.
- `bbs_verify_signature` verifies a signature over the same raw byte messages.
- `bbs_create_proof` creates a proof of knowledge of a signature with selected message indexes revealed.
- `bbs_verify_proof` verifies the proof using the revealed indexed raw messages.
- `bbs_free_buffer` and `bbs_free_keypair` release buffers allocated by the library.

All serialized cryptographic values use Dock/ark canonical compressed bytes. The FFI boundary uses plain pointers, lengths, and integer status codes for straightforward cgo usage.

## Build Locally

Run the Rust checks on your host:

```sh
make test
```

Build and test the Linux AMD64 Rust and Go package in Docker:

```sh
make docker-ci
```

Build the ready-to-use Linux AMD64 artifact:

```sh
make docker-artifacts
```

Build and test the Apple Silicon macOS package on an M-series Mac:

```sh
make sync-go-native-darwin-arm64
make test-go-ffi
make package-darwin-arm64
```

The Linux artifact is written to:

```text
dist/libbbsplus-linux-amd64.tar.gz
```

The Apple Silicon artifact is written to:

```text
dist/libbbsplus-darwin-arm64.tar.gz
```

Each artifact contains:

```text
libbbsplus-<os>-<arch>/
  include/bbs_ffi.h
  lib/libbbsplus.a
```

## Go Usage

On Linux AMD64 or Apple Silicon macOS with cgo enabled, use the Go wrapper module from this repository without any extra cgo flags:

```go
package main

import bbs "github.com/Grunt-MVT/bbs/go"

func main() {
	messages := [][]byte{
		[]byte("alice"),
		[]byte("US"),
		[]byte("1716328800"),
	}

	keyPair, err := bbs.GenerateKeyPair(len(messages))
	if err != nil {
		panic(err)
	}

	signature, err := bbs.Sign(keyPair.Params, keyPair.SecretKey, messages)
	if err != nil {
		panic(err)
	}

	if err := bbs.VerifySignature(keyPair.Params, keyPair.PublicKey, messages, signature); err != nil {
		panic(err)
	}
}
```

The package includes its own cgo directives and bundled static native archive. A normal Go build is enough:

```sh
go test ./...
```

Because the native code is linked statically into the Go binary, there is no `libbbsplus.so` or `libbbsplus.dylib` runtime lookup.

This is still a cgo package, so `CGO_ENABLED=1` and a C linker are required. The bundled native archives currently target Linux AMD64 and Apple Silicon macOS (`darwin/arm64`); other platforms need their own `go/native/<os>_<arch>` archive and cgo directives.

See [`go`](go) for the wrapper package and an end-to-end test covering key generation, signing, signature verification, proof creation, and proof verification.
