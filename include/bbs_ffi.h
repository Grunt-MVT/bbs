#ifndef BBS_FFI_H
#define BBS_FFI_H

#include <stdint.h>
#include <stddef.h>

#ifdef __cplusplus
extern "C" {
#endif

#define BBS_OK 0
#define BBS_ERROR_NULL_POINTER 1
#define BBS_ERROR_INVALID_LENGTH 2
#define BBS_ERROR_INVALID_INDEX 3
#define BBS_ERROR_DESERIALIZE 4
#define BBS_ERROR_SERIALIZE 5
#define BBS_ERROR_CRYPTO 6
#define BBS_ERROR_VERIFY_FAILED 7
#define BBS_ERROR_TOO_MANY_MSGS 8
#define BBS_ERROR_PANIC 255

typedef struct BbsByteSlice {
  const uint8_t *data;
  size_t len;
} BbsByteSlice;

typedef struct BbsByteBuffer {
  uint8_t *data;
  size_t len;
} BbsByteBuffer;

typedef struct BbsMessage {
  const uint8_t *data;
  size_t len;
} BbsMessage;

typedef struct BbsIndexedMessage {
  uint32_t index;
  const uint8_t *data;
  size_t len;
} BbsIndexedMessage;

typedef struct BbsKeyPair {
  BbsByteBuffer params;
  BbsByteBuffer secret_key;
  BbsByteBuffer public_key;
} BbsKeyPair;

// Frees a byte buffer returned by this library.
void bbs_free_buffer(BbsByteBuffer buffer);

// Frees the params, secret key, and public key buffers returned by bbs_generate_keypair.
void bbs_free_keypair(BbsKeyPair *keypair);

// Returns a static null-terminated string describing a status code.
const char *bbs_status_message(int32_t status);

// Generates Dock-compatible signature params and a BBS+ keypair for message_count messages.
int32_t bbs_generate_keypair(uint32_t message_count, BbsKeyPair *out_keypair);

// Signs raw byte messages. If fewer messages are supplied than the params support, missing
// slots are padded internally before messages are hashed by zero-based position.
int32_t bbs_sign(
    BbsByteSlice params,
    BbsByteSlice secret_key,
    const BbsMessage *messages,
    size_t message_count,
    BbsByteBuffer *out_signature);

// Verifies a signature over raw byte messages using the same padding and internal hashing as
// bbs_sign.
int32_t bbs_verify_signature(
    BbsByteSlice params,
    BbsByteSlice public_key,
    const BbsMessage *messages,
    size_t message_count,
    BbsByteSlice signature);

// Creates a selective-disclosure proof. Missing message slots are padded internally, and
// revealed_indices are zero-based indexes in the padded message vector.
int32_t bbs_create_proof(
    BbsByteSlice params,
    BbsByteSlice public_key,
    BbsByteSlice signature,
    const BbsMessage *messages,
    size_t message_count,
    const uint32_t *revealed_indices,
    size_t revealed_indices_count,
    BbsByteBuffer *out_proof);

// Verifies a selective-disclosure proof against the disclosed indexed raw messages.
int32_t bbs_verify_proof(
    BbsByteSlice params,
    BbsByteSlice public_key,
    BbsByteSlice proof,
    const BbsIndexedMessage *revealed_messages,
    size_t revealed_message_count);

#ifdef __cplusplus
}
#endif

#endif
