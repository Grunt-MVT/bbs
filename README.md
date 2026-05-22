# libbbsplus

`libbbsplus` is a small FFI wrapper over Dock's BBS+ Rust library. It exposes a C ABI so services written in Go, or another language with C FFI support, can generate keys, sign and verify messages, and create and verify selective-disclosure proofs without binding directly to Dock's Rust types.

The shared library is built as `libbbsplus.so` on Linux. Public functions keep the `bbs_` prefix and are declared in [`include/bbs_ffi.h`](include/bbs_ffi.h).

## What It Exposes

- `bbs_generate_keypair` creates Dock-compatible signature parameters, secret key, and public key bytes.
- `bbs_sign` signs raw byte messages. Messages are hashed to BLS12-381 field elements inside the library.
- `bbs_verify_signature` verifies a signature over the same raw byte messages.
- `bbs_create_proof` creates a proof of knowledge of a signature with selected message indexes revealed.
- `bbs_verify_proof` verifies the proof using the revealed indexed raw messages.
- `bbs_free_buffer` and `bbs_free_keypair` release buffers allocated by the library.

All serialized cryptographic values use Dock/ark canonical compressed bytes. The FFI boundary uses plain pointers, lengths, and integer status codes for straightforward cgo usage.

## Build Locally

Run the Rust and Go FFI checks on your host:

```sh
make ci
```

Build and test in Docker for the Linux AMD64 target:

```sh
make docker-ci
```

Build the ready-to-use Linux AMD64 artifact:

```sh
make docker-artifacts
```

The artifact is written to:

```text
dist/libbbsplus-linux-amd64.tar.gz
```

It contains:

```text
libbbsplus-linux-amd64/
  include/bbs_ffi.h
  lib/libbbsplus.so
```

## Go Usage

Link Go code with cgo against the header and shared library:

```go
/*
#cgo CFLAGS: -I/path/to/libbbsplus/include
#cgo LDFLAGS: -L/path/to/libbbsplus/lib -lbbsplus
#include "bbs_ffi.h"
*/
import "C"
```

At runtime, make sure the dynamic linker can find `libbbsplus.so`, for example:

```sh
LD_LIBRARY_PATH=/path/to/libbbsplus/lib go test ./...
```

See [`tests/goffi`](tests/goffi) for an end-to-end Go test that calls key generation, signing, signature verification, proof creation, proof verification, and cleanup through the FFI API.
