//go:build !cgo || (!linux && !darwin) || (linux && !amd64) || (darwin && !arm64)

package bbs

import (
	"errors"
	"fmt"
)

const pidOrderLen = 5

// MaxMessageCount is the protocol message capacity used for key generation and padding.
const MaxMessageCount = 20

const (
	// StatusOK is returned by the native library when an operation succeeds.
	StatusOK int32 = 0

	// StatusVerifyFailed is returned when a signature or proof does not verify.
	StatusVerifyFailed int32 = 7

	// StatusTooManyMessages is returned when more messages are supplied than params support.
	StatusTooManyMessages int32 = 8
)

// PIDOrder contains PID identifiers as defined in
// https://eudi.dev/2.4.0/annexes/annex-3/annex-3.01-pid-rulebook/.
var PIDOrder = [pidOrderLen]string{
	"family_name",
	"given_name",
	"birth_date",
	"birth_place",
	"nationality",
}

var errUnsupported = errors.New("libbbsplus: bundled native library is available only on linux/amd64 or darwin/arm64 with cgo enabled")

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
	return nil, errUnsupported
}

// Sign signs raw byte messages. Missing slots are padded by libbbsplus to MaxMessageCount.
func Sign(secretKey []byte, messages [][]byte) ([]byte, error) {
	return nil, errUnsupported
}

// VerifySignature verifies a BBS+ signature over raw byte messages.
func VerifySignature(publicKey []byte, messages [][]byte, signature []byte) error {
	return errUnsupported
}

// CreateProof creates a selective-disclosure proof for a BBS+ signature.
func CreateProof(publicKey, signature []byte, messages [][]byte, revealed []uint32) ([]byte, error) {
	return nil, errUnsupported
}

// VerifyProof verifies a selective-disclosure proof against the revealed raw messages.
func VerifyProof(publicKey, proof []byte, revealed []RevealedMessage) error {
	return errUnsupported
}
