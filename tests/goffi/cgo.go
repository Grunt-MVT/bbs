package goffi

/*
#include "bbs_ffi.h"
#include <stdlib.h>
*/
import "C"

import "unsafe"

type status int32

const (
	statusOK           status = C.BBS_OK
	statusVerifyFailed status = C.BBS_ERROR_VERIFY_FAILED
)

type keypair struct {
	raw C.BbsKeyPair
}

type buffer struct {
	raw C.BbsByteBuffer
}

type indexedMessage struct {
	index uint32
	data  []byte
}

func statusString(code status) string {
	return C.GoString(C.bbs_status_message(C.int32_t(code)))
}

func generateKeypair(messageCount int) (*keypair, status) {
	kp := &keypair{}
	code := C.bbs_generate_keypair(C.uint32_t(messageCount), &kp.raw)
	if status(code) != statusOK {
		return nil, status(code)
	}
	return kp, statusOK
}

func (kp *keypair) free() {
	if kp != nil {
		C.bbs_free_keypair(&kp.raw)
	}
}

func (buf *buffer) free() {
	if buf != nil {
		C.bbs_free_buffer(buf.raw)
	}
}

func byteSlice(buf C.BbsByteBuffer) C.BbsByteSlice {
	return C.BbsByteSlice{
		data: (*C.uint8_t)(buf.data),
		len:  buf.len,
	}
}

func cBytes(data []byte) unsafe.Pointer {
	if len(data) == 0 {
		return nil
	}
	return C.CBytes(data)
}

func cMessages(messages [][]byte) (*C.BbsMessage, func()) {
	if len(messages) == 0 {
		return nil, func() {}
	}

	raw := C.malloc(C.size_t(len(messages)) * C.size_t(unsafe.Sizeof(C.BbsMessage{})))
	out := unsafe.Slice((*C.BbsMessage)(raw), len(messages))
	allocations := make([]unsafe.Pointer, len(messages))

	for i, msg := range messages {
		allocations[i] = cBytes(msg)
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

func cIndexedMessages(messages []indexedMessage) (*C.BbsIndexedMessage, func()) {
	if len(messages) == 0 {
		return nil, func() {}
	}

	raw := C.malloc(C.size_t(len(messages)) * C.size_t(unsafe.Sizeof(C.BbsIndexedMessage{})))
	out := unsafe.Slice((*C.BbsIndexedMessage)(raw), len(messages))
	allocations := make([]unsafe.Pointer, len(messages))

	for i, msg := range messages {
		allocations[i] = cBytes(msg.data)
		out[i] = C.BbsIndexedMessage{
			index: C.uint32_t(msg.index),
			data:  (*C.uint8_t)(allocations[i]),
			len:   C.size_t(len(msg.data)),
		}
	}

	return (*C.BbsIndexedMessage)(raw), func() {
		for _, allocation := range allocations {
			C.free(allocation)
		}
		C.free(raw)
	}
}

func sign(kp *keypair, messages [][]byte) (*buffer, status) {
	ffiMsgs, freeMessages := cMessages(messages)
	defer freeMessages()

	sig := &buffer{}
	code := C.bbs_sign(
		byteSlice(kp.raw.params),
		byteSlice(kp.raw.secret_key),
		ffiMsgs,
		C.size_t(len(messages)),
		&sig.raw,
	)
	if status(code) != statusOK {
		return nil, status(code)
	}
	return sig, statusOK
}

func verifySignature(kp *keypair, sig *buffer, messages [][]byte) status {
	ffiMsgs, freeMessages := cMessages(messages)
	defer freeMessages()

	return status(C.bbs_verify_signature(
		byteSlice(kp.raw.params),
		byteSlice(kp.raw.public_key),
		ffiMsgs,
		C.size_t(len(messages)),
		byteSlice(sig.raw),
	))
}

func createProof(kp *keypair, sig *buffer, messages [][]byte, revealed []uint32) (*buffer, status) {
	ffiMsgs, freeMessages := cMessages(messages)
	defer freeMessages()

	ffiRevealed := make([]C.uint32_t, len(revealed))
	for i, index := range revealed {
		ffiRevealed[i] = C.uint32_t(index)
	}

	proof := &buffer{}
	code := C.bbs_create_proof(
		byteSlice(kp.raw.params),
		byteSlice(kp.raw.public_key),
		byteSlice(sig.raw),
		ffiMsgs,
		C.size_t(len(messages)),
		&ffiRevealed[0],
		C.size_t(len(ffiRevealed)),
		&proof.raw,
	)
	if status(code) != statusOK {
		return nil, status(code)
	}
	return proof, statusOK
}

func verifyProof(kp *keypair, proof *buffer, revealed []indexedMessage) status {
	ffiRevealed, freeRevealed := cIndexedMessages(revealed)
	defer freeRevealed()

	return status(C.bbs_verify_proof(
		byteSlice(kp.raw.params),
		byteSlice(kp.raw.public_key),
		byteSlice(proof.raw),
		ffiRevealed,
		C.size_t(len(revealed)),
	))
}
