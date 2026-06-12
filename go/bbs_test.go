//go:build ((linux && amd64) || (darwin && arm64)) && cgo

package bbs_test

import (
	"testing"

	bbs "github.com/Grunt-MVT/bbs/go"
)

func requireNoError(t *testing.T, err error) {
	t.Helper()
	if err != nil {
		t.Fatal(err)
	}
}

func requireStatus(t *testing.T, err error, want int32) {
	t.Helper()

	got, ok := bbs.StatusCode(err)
	if !ok {
		t.Fatalf("expected libbbsplus status %d, got %v", want, err)
	}
	if got != want {
		t.Fatalf("expected libbbsplus status %d, got %d (%v)", want, got, err)
	}
}

func TestEndToEnd(t *testing.T) {
	messages := [][]byte{
		[]byte("alice"),
		[]byte("US"),
		[]byte("1716328800"),
	}

	keyPair, err := bbs.GenerateKeyPair(len(messages))
	requireNoError(t, err)

	signature, err := bbs.Sign(keyPair.Params, keyPair.SecretKey, messages)
	requireNoError(t, err)

	err = bbs.VerifySignature(keyPair.Params, keyPair.PublicKey, messages, signature)
	requireNoError(t, err)

	badMessages := [][]byte{
		[]byte("alice"),
		[]byte("CA"),
		[]byte("1716328800"),
	}
	err = bbs.VerifySignature(keyPair.Params, keyPair.PublicKey, badMessages, signature)
	requireStatus(t, err, bbs.StatusVerifyFailed)

	proof, err := bbs.CreateProof(
		keyPair.Params,
		keyPair.PublicKey,
		signature,
		messages,
		[]uint32{0, 2},
	)
	requireNoError(t, err)

	err = bbs.VerifyProof(keyPair.Params, keyPair.PublicKey, proof, []bbs.RevealedMessage{
		{Index: 0, Data: messages[0]},
		{Index: 2, Data: messages[2]},
	})
	requireNoError(t, err)

	err = bbs.VerifyProof(keyPair.Params, keyPair.PublicKey, proof, []bbs.RevealedMessage{
		{Index: 0, Data: messages[0]},
		{Index: 2, Data: []byte("1716328801")},
	})
	requireStatus(t, err, bbs.StatusVerifyFailed)
}

func TestNativePadding(t *testing.T) {
	messages := [][]byte{
		[]byte("alice"),
		[]byte("US"),
		[]byte("active"),
	}

	keyPair, err := bbs.GenerateKeyPair(20)
	requireNoError(t, err)

	signature, err := bbs.Sign(keyPair.Params, keyPair.SecretKey, messages)
	requireNoError(t, err)

	err = bbs.VerifySignature(keyPair.Params, keyPair.PublicKey, messages, signature)
	requireNoError(t, err)

	proof, err := bbs.CreateProof(
		keyPair.Params,
		keyPair.PublicKey,
		signature,
		messages,
		[]uint32{0, 2},
	)
	requireNoError(t, err)

	err = bbs.VerifyProof(keyPair.Params, keyPair.PublicKey, proof, []bbs.RevealedMessage{
		{Index: 0, Data: messages[0]},
		{Index: 2, Data: messages[2]},
	})
	requireNoError(t, err)

	_, err = bbs.Sign(keyPair.Params, keyPair.SecretKey, make([][]byte, 21))
	requireStatus(t, err, bbs.StatusTooManyMessages)
}

func TestPIDOrder(t *testing.T) {
	want := [5]string{
		"family_name",
		"given_name",
		"birth_date",
		"birth_place",
		"nationality",
	}

	if bbs.PIDOrder != want {
		t.Fatalf("unexpected PID order: got %v, want %v", bbs.PIDOrder, want)
	}
}
