# libbbsplus

`libbbsplus` is a small FFI wrapper over Dock's BBS+ Rust library. It exposes a **C ABI** as the stable native boundary. Go and Node bindings are thin adapters over that ABI—they do not reimplement cryptography.

```text
                 Rust BBS implementation
                            │
                            ▼
                     C ABI (bbs_ffi.h)
                       libbbsplus.a
                            │
               ┌────────────┴────────────┐
               ▼                         ▼
          Go / cgo                  Node N-API
               │                         │
               ▼                         ▼
          Go package              bbsplus_node.node
```

The expensive BBS core is compiled **once per OS/arch** into `libbbsplus.a`. Go links that archive via cgo; the Node addon statically links the same archive into a self-contained `.node` file.

The Go package vendors static native archives for Linux AMD64 and Apple Silicon macOS. The Node package vendors matching prebuilt addons for the same targets. Public native functions keep the `bbs_` prefix and are declared in [`include/bbs_ffi.h`](include/bbs_ffi.h).

## What It Exposes

- `bbs_generate_keypair` creates Dock-compatible signature parameters, secret key, and public key bytes for a maximum/common message count.
- `bbs_sign` signs raw byte messages. If fewer messages are supplied than the params support, the library pads missing slots internally before hashing messages to BLS12-381 field elements.
- `bbs_verify_signature` verifies a signature over the same raw byte messages using the same padding and hashing rules.
- `bbs_create_proof` creates a proof of knowledge of a signature with selected message indexes revealed.
- `bbs_verify_proof` verifies the proof using the revealed indexed raw messages.
- `bbs_pid_order` exposes the PID identifier order used by the wrapper.
- `bbs_free_buffer` and `bbs_free_keypair` release buffers allocated by the library.

All serialized cryptographic values use Dock/ark canonical compressed bytes. The FFI boundary uses plain pointers, lengths, and integer status codes for straightforward cgo usage.

Signature params are deterministic and derived internally from a fixed API identifier and `MAX_MESSAGE_COUNT` (20). Callers do not pass params into sign, verify, or proof functions. `bbs_generate_keypair` still returns serialized params for storage or inspection, but `message_count` must equal 20.

Signature params define the padded message-vector length. This lets applications use a common length of 20 slots while signing credentials that currently fill fewer slots. Missing slots are padded in Rust with an internal invalid-UTF-8 sentinel (`0xff || "bbsplus-padding-v1" || 0xfe`). Supplying more than 20 messages returns `BBS_ERROR_TOO_MANY_MSGS`.

Proof indexes always refer to the padded vector layout. Applications should keep stable slot ordering, for example slot 0 = name, slot 1 = country, slot 2 = expiration, and so on.

## Build Locally

Run the Rust checks on your host:

```sh
make test
```

Build the shared core once for the host target:

```sh
make build-release
```

That produces `target/release/libbbsplus.a` (and the accompanying rlib used only by Rust tests).

### Darwin ARM64

```sh
make sync-go-native-darwin-arm64   # core → go/native/darwin_arm64
make test-go-ffi
make sync-node-native-darwin-arm64 # links core into node/native/darwin_arm64/bbsplus_node.node
make package-darwin-arm64
```

### Linux AMD64

Preferred path in CI/Docker (glibc / Debian bookworm):

```sh
make docker-ci
make docker-artifacts
make docker-sync-go-native-linux-amd64
```

On a Linux AMD64 host:

```sh
make sync-go-native-linux-amd64
make test-go-ffi
make sync-node-native-linux-amd64
make package-linux-amd64
```

**musl:** not used. Targets are glibc-based (Debian bookworm in Docker, typical Ubuntu/AL Node hosts). Replacing glibc builds with musl would break loading under glibc Node runtimes (including common Vercel/AWS Linux environments). A separate musl target could be added later if needed; it should not replace the glibc Linux artifact.

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

	keyPair, err := bbs.GenerateKeyPair(bbs.MaxMessageCount)
	if err != nil {
		panic(err)
	}

	signature, err := bbs.Sign(keyPair.SecretKey, messages)
	if err != nil {
		panic(err)
	}

	if err := bbs.VerifySignature(keyPair.PublicKey, messages, signature); err != nil {
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

The Node package exposes protocol metadata, string canonicalization helpers, and proof verification:

```ts
import {
  maxMessageCount,
  pidOrder,
  canonicalString,
  canonicalNationality,
  canonicalNationalityList,
  verifyProof,
} from "@lithosid/bbs-node";

const ok = verifyProof(publicKeyBytes, proofBytes, [
  { index: 0, data: Buffer.from("Doe") },
  { index: 2, data: Buffer.from("1990-01-01") },
]);

canonicalNationalityList(["it", "pl", "cz "]); // "CZITPL"
```

`pidOrder` is the ordered PID identifier list:

```ts
[
  "family_name",
  "given_name",
  "birth_date",
  "birth_place",
  "nationality",
  "derived_nationality",
]
```

`canonicalString` trims and uppercases. `canonicalNationality` requires exactly two ASCII letters `A`–`Z` after that normalization. `canonicalNationalityList` applies the same rule to each entry, sorts, and concatenates.

All Node crypto inputs are raw bytes (`Uint8Array` or `Buffer`). Store public keys and proofs with binary-safe encodings such as base64 before placing them in JSON, environment variables, or secret vaults. Proof verification uses the same fixed `maxMessageCount` (20) as the Go package.

The package vendors prebuilt N-API addons for Apple Silicon macOS (`darwin/arm64`) and Linux AMD64 (`linux/amd64`). At load time [`node/src/index.ts`](node/src/index.ts) selects `native/<os>_<arch>/bbsplus_node.node` from `process.platform` and `process.arch`, so a macOS build cannot be loaded on Linux. Other platforms throw a clear unsupported-platform error.

The N-API addon is a thin **C++** adapter (`node-addon-api`): it translates JS values to C ABI types and statically links `libbbsplus.a` (from `make build-release`). It does not recompile the BBS Rust implementation and does not embed a second Rust runtime.

Build and test the Node package locally (builds the shared core first, then the addon):

```sh
make test-node
```
