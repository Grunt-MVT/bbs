package goffi

import "testing"

func requireOK(t *testing.T, code status) {
	t.Helper()
	if code != statusOK {
		t.Fatalf("expected BBS_OK, got %d (%s)", int32(code), statusString(code))
	}
}

func TestBbsFfiEndToEnd(t *testing.T) {
	messages := [][]byte{
		[]byte("alice"),
		[]byte("US"),
		[]byte("1716328800"),
	}

	keypair, code := generateKeypair(len(messages))
	requireOK(t, code)
	defer keypair.free()

	signature, code := sign(keypair, messages)
	requireOK(t, code)
	defer signature.free()

	requireOK(t, verifySignature(keypair, signature, messages))

	badMessages := [][]byte{
		[]byte("alice"),
		[]byte("CA"),
		[]byte("1716328800"),
	}
	code = verifySignature(keypair, signature, badMessages)
	if code != statusVerifyFailed {
		t.Fatalf("expected signature verification failure, got %d (%s)", int32(code), statusString(code))
	}

	proof, code := createProof(keypair, signature, messages, []uint32{0, 2})
	requireOK(t, code)
	defer proof.free()

	revealedMessages := []indexedMessage{
		{index: 0, data: messages[0]},
		{index: 2, data: messages[2]},
	}
	requireOK(t, verifyProof(keypair, proof, revealedMessages))

	tamperedRevealedMessages := []indexedMessage{
		{index: 0, data: messages[0]},
		{index: 2, data: []byte("1716328801")},
	}
	code = verifyProof(keypair, proof, tamperedRevealedMessages)
	if code != statusVerifyFailed {
		t.Fatalf("expected proof verification failure, got %d (%s)", int32(code), statusString(code))
	}
}
