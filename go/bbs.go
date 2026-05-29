//go:build linux && amd64 && cgo

// Package bbs provides a Go wrapper for libbbsplus.
package bbs

/*
#cgo linux,amd64 CFLAGS: -I${SRCDIR}/native/linux_amd64/include
#cgo linux,amd64 LDFLAGS: ${SRCDIR}/native/linux_amd64/lib/libbbsplus.a -ldl -lm -lpthread
#include "bbs_ffi.h"
#include <stdlib.h>
*/
import "C"

import (
	"errors"
	"fmt"
	"math"
	"unsafe"
)

const (
	// StatusOK is returned by the native library when an operation succeeds.
	StatusOK int32 = C.BBS_OK

	// StatusVerifyFailed is returned when a signature or proof does not verify.
	StatusVerifyFailed int32 = C.BBS_ERROR_VERIFY_FAILED
)

// Error wraps a non-OK status returned by libbbsplus.
type Error struct {
	Code    int32
	Message string
}

func (err *Error) Error() string {
	return fmt.Sprintf("libbbsplus: %s (status %d)", err.Message, err.Code)
}

// KeyPair contains Dock-compatible canonical compressed bytes.
type KeyPair struct {
	Params    []byte
	SecretKey []byte
	PublicKey []byte
}

// RevealedMessage is a raw message disclosed during proof verification.
type RevealedMessage struct {
	Index uint32
	Data  []byte
}

// StatusCode returns the native libbbsplus status code stored in err.
func StatusCode(err error) (int32, bool) {
	var bbsErr *Error
	if errors.As(err, &bbsErr) {
		return bbsErr.Code, true
	}
	return 0, false
}

// GenerateKeyPair creates signature parameters and a BBS+ keypair for messageCount messages.
func GenerateKeyPair(messageCount int) (*KeyPair, error) {
	if messageCount <= 0 || messageCount > math.MaxUint32 {
		return nil, fmt.Errorf("libbbsplus: invalid message count %d", messageCount)
	}

	var raw C.BbsKeyPair
	code := C.bbs_generate_keypair(C.uint32_t(messageCount), &raw)
	if err := statusError(code); err != nil {
		return nil, err
	}
	defer C.bbs_free_keypair(&raw)

	return &KeyPair{
		Params:    bytesFromBuffer(raw.params),
		SecretKey: bytesFromBuffer(raw.secret_key),
		PublicKey: bytesFromBuffer(raw.public_key),
	}, nil
}

// Sign signs raw byte messages. Messages are hashed to field elements by libbbsplus.
func Sign(params, secretKey []byte, messages [][]byte) ([]byte, error) {
	cParams, freeParams := cByteSlice(params)
	defer freeParams()
	cSecretKey, freeSecretKey := cByteSlice(secretKey)
	defer freeSecretKey()
	cMessages, freeMessages := cMessages(messages)
	defer freeMessages()

	var signature C.BbsByteBuffer
	code := C.bbs_sign(
		cParams,
		cSecretKey,
		cMessages,
		C.size_t(len(messages)),
		&signature,
	)
	if err := statusError(code); err != nil {
		return nil, err
	}
	defer C.bbs_free_buffer(signature)

	return bytesFromBuffer(signature), nil
}

// VerifySignature verifies a BBS+ signature over raw byte messages.
func VerifySignature(params, publicKey []byte, messages [][]byte, signature []byte) error {
	cParams, freeParams := cByteSlice(params)
	defer freeParams()
	cPublicKey, freePublicKey := cByteSlice(publicKey)
	defer freePublicKey()
	cSignature, freeSignature := cByteSlice(signature)
	defer freeSignature()
	cMessages, freeMessages := cMessages(messages)
	defer freeMessages()

	return statusError(C.bbs_verify_signature(
		cParams,
		cPublicKey,
		cMessages,
		C.size_t(len(messages)),
		cSignature,
	))
}

// CreateProof creates a selective-disclosure proof for a BBS+ signature.
func CreateProof(params, publicKey, signature []byte, messages [][]byte, revealed []uint32) ([]byte, error) {
	cParams, freeParams := cByteSlice(params)
	defer freeParams()
	cPublicKey, freePublicKey := cByteSlice(publicKey)
	defer freePublicKey()
	cSignature, freeSignature := cByteSlice(signature)
	defer freeSignature()
	cMessages, freeMessages := cMessages(messages)
	defer freeMessages()
	cRevealed, freeRevealed := cUint32s(revealed)
	defer freeRevealed()

	var proof C.BbsByteBuffer
	code := C.bbs_create_proof(
		cParams,
		cPublicKey,
		cSignature,
		cMessages,
		C.size_t(len(messages)),
		cRevealed,
		C.size_t(len(revealed)),
		&proof,
	)
	if err := statusError(code); err != nil {
		return nil, err
	}
	defer C.bbs_free_buffer(proof)

	return bytesFromBuffer(proof), nil
}

// VerifyProof verifies a selective-disclosure proof against the revealed raw messages.
func VerifyProof(params, publicKey, proof []byte, revealed []RevealedMessage) error {
	cParams, freeParams := cByteSlice(params)
	defer freeParams()
	cPublicKey, freePublicKey := cByteSlice(publicKey)
	defer freePublicKey()
	cProof, freeProof := cByteSlice(proof)
	defer freeProof()
	cRevealed, freeRevealed := cRevealedMessages(revealed)
	defer freeRevealed()

	return statusError(C.bbs_verify_proof(
		cParams,
		cPublicKey,
		cProof,
		cRevealed,
		C.size_t(len(revealed)),
	))
}

func statusError(code C.int32_t) error {
	if code == C.BBS_OK {
		return nil
	}
	return &Error{
		Code:    int32(code),
		Message: C.GoString(C.bbs_status_message(code)),
	}
}

func bytesFromBuffer(buffer C.BbsByteBuffer) []byte {
	if buffer.data == nil || buffer.len == 0 {
		return nil
	}
	return C.GoBytes(unsafe.Pointer(buffer.data), C.int(buffer.len))
}

func cByteSlice(data []byte) (C.BbsByteSlice, func()) {
	if len(data) == 0 {
		return C.BbsByteSlice{}, func() {}
	}

	raw := C.CBytes(data)
	return C.BbsByteSlice{
			data: (*C.uint8_t)(raw),
			len:  C.size_t(len(data)),
		}, func() {
			C.free(raw)
		}
}

func cMessages(messages [][]byte) (*C.BbsMessage, func()) {
	if len(messages) == 0 {
		return nil, func() {}
	}

	raw := C.malloc(C.size_t(len(messages)) * C.size_t(unsafe.Sizeof(C.BbsMessage{})))
	out := unsafe.Slice((*C.BbsMessage)(raw), len(messages))
	allocations := make([]unsafe.Pointer, len(messages))

	for i, msg := range messages {
		if len(msg) != 0 {
			allocations[i] = C.CBytes(msg)
		}
		out[i] = C.BbsMessage{
			data: (*C.uint8_t)(allocations[i]),
			len:  C.size_t(len(msg)),
		}
	}

	return (*C.BbsMessage)(raw), func() {
		for _, allocation := range allocations {
			C.free(allocation)
		}
		C.free(raw)
	}
}

func cRevealedMessages(messages []RevealedMessage) (*C.BbsIndexedMessage, func()) {
	if len(messages) == 0 {
		return nil, func() {}
	}

	raw := C.malloc(C.size_t(len(messages)) * C.size_t(unsafe.Sizeof(C.BbsIndexedMessage{})))
	out := unsafe.Slice((*C.BbsIndexedMessage)(raw), len(messages))
	allocations := make([]unsafe.Pointer, len(messages))

	for i, msg := range messages {
		if len(msg.Data) != 0 {
			allocations[i] = C.CBytes(msg.Data)
		}
		out[i] = C.BbsIndexedMessage{
			index: C.uint32_t(msg.Index),
			data:  (*C.uint8_t)(allocations[i]),
			len:   C.size_t(len(msg.Data)),
		}
	}

	return (*C.BbsIndexedMessage)(raw), func() {
		for _, allocation := range allocations {
			C.free(allocation)
		}
		C.free(raw)
	}
}

func cUint32s(values []uint32) (*C.uint32_t, func()) {
	if len(values) == 0 {
		return nil, func() {}
	}

	raw := C.malloc(C.size_t(len(values)) * C.size_t(unsafe.Sizeof(C.uint32_t(0))))
	out := unsafe.Slice((*C.uint32_t)(raw), len(values))
	for i, value := range values {
		out[i] = C.uint32_t(value)
	}

	return (*C.uint32_t)(raw), func() {
		C.free(raw)
	}
}
