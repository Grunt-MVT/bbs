# libbbsplus

`libbbsplus` is a small FFI wrapper over Dock's BBS+ Rust library. It exposes a C ABI and a Go package so services written in Go can generate keys, sign and verify messages, and create and verify selective-disclosure proofs without binding directly to Dock's Rust types.

The Go package vendors static native archives for Linux AMD64 and Apple Silicon macOS, so users on those targets do not need to pass cgo compiler/linker flags or configure runtime library paths. Public native functions keep the `bbs_` prefix and are declared in [`include/bbs_ffi.h`](include/bbs_ffi.h).

## What It Exposes

- `bbs_generate_keypair` creates Dock-compatible signature parameters, secret key, and public key bytes for a maximum/common message count.
- `bbs_sign` signs raw byte messages. If fewer messages are supplied than the params support, the library pads missing slots internally before hashing messages to BLS12-381 field elements.
- `bbs_verify_signature` verifies a signature over the same raw byte messages using the same padding and hashing rules.
- `bbs_create_proof` creates a proof of knowledge of a signature with selected message indexes revealed.
- `bbs_verify_proof` verifies the proof using the revealed indexed raw messages.
- `bbs_pid_order` exposes the PID identifier order used by the wrapper.
- `bbs_free_buffer` and `bbs_free_keypair` release buffers allocated by the library.

All serialized cryptographic values use Dock/ark canonical compressed bytes. The FFI boundary uses plain pointers, lengths, and integer status codes for straightforward cgo usage.

Signature params define the padded message-vector length. This lets applications use a common length, such as 20 slots, while signing credentials that currently fill fewer slots. Missing slots are padded in Rust with an internal invalid-UTF-8 sentinel (`0xff || "bbsplus-padding-v1" || 0xfe`). Supplying more messages than the params support returns `BBS_ERROR_TOO_MANY_MSGS`.

Proof indexes always refer to the padded vector layout. Applications should keep stable slot ordering, for example slot 0 = name, slot 1 = country, slot 2 = expiration, and so on.

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
	const maxMessages = 20

	messages := [][]byte{
		[]byte("alice"),
		[]byte("US"),
		[]byte("1716328800"),
	}

	keyPair, err := bbs.GenerateKeyPair(maxMessages)
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

## Node/TypeScript Usage

The Node package exposes only protocol metadata and proof verification:

```ts
import { pidOrder, verifyProof } from "@grunt-mvt/bbs-node";

const ok = verifyProof(paramsBytes, publicKeyBytes, proofBytes, [
  { index: 0, data: Buffer.from("Doe") },
  { index: 2, data: Buffer.from("1990-01-01") },
]);
```

`pidOrder` is the ordered PID identifier list:

```ts
[
  "family_name",
  "given_name",
  "birth_date",
  "birth_place",
  "nationality",
]
```

All Node inputs are raw bytes (`Uint8Array` or `Buffer`). Store params, public keys, and proofs with binary-safe encodings such as base64 before placing them in JSON, environment variables, or secret vaults.

Build and test the Node package locally:

```sh
make test-node
```
