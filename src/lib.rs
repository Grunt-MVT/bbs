use std::collections::{BTreeMap, BTreeSet};
use std::ffi::c_char;
use std::os::raw::c_int;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::ptr;
use std::slice;

use ark_bls12_381::{Bls12_381, Fr};
use ark_serialize::{CanonicalDeserialize, CanonicalSerialize};
use bbs_plus::{
    proof::{PoKOfSignatureG1Proof, PoKOfSignatureG1Protocol},
    setup::{KeypairG2, PublicKeyG2, SecretKey, SignatureParamsG1},
    signature::SignatureG1,
};
use blake2::Blake2b512;
use dock_crypto_utils::hashing_utils::hash_to_field;
use dock_crypto_utils::signature::MessageOrBlinding;
use rand::rngs::OsRng;
use schnorr_pok::compute_random_oracle_challenge;

type Params = SignatureParamsG1<Bls12_381>;
type PublicKey = PublicKeyG2<Bls12_381>;
type Secret = SecretKey<Fr>;
type Signature = SignatureG1<Bls12_381>;
type Proof = PoKOfSignatureG1Proof<Bls12_381>;

const MESSAGE_DOMAIN_PREFIX: &[u8] = b"bbs-ffi/v1/message/";

pub const BBS_OK: c_int = 0;
pub const BBS_ERROR_NULL_POINTER: c_int = 1;
pub const BBS_ERROR_INVALID_LENGTH: c_int = 2;
pub const BBS_ERROR_INVALID_INDEX: c_int = 3;
pub const BBS_ERROR_DESERIALIZE: c_int = 4;
pub const BBS_ERROR_SERIALIZE: c_int = 5;
pub const BBS_ERROR_CRYPTO: c_int = 6;
pub const BBS_ERROR_VERIFY_FAILED: c_int = 7;
pub const BBS_ERROR_PANIC: c_int = 255;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
enum FfiError {
    NullPointer,
    InvalidLength,
    InvalidIndex,
    Deserialize,
    Serialize,
    Crypto,
    VerifyFailed,
}

impl FfiError {
    fn code(self) -> c_int {
        match self {
            FfiError::NullPointer => BBS_ERROR_NULL_POINTER,
            FfiError::InvalidLength => BBS_ERROR_INVALID_LENGTH,
            FfiError::InvalidIndex => BBS_ERROR_INVALID_INDEX,
            FfiError::Deserialize => BBS_ERROR_DESERIALIZE,
            FfiError::Serialize => BBS_ERROR_SERIALIZE,
            FfiError::Crypto => BBS_ERROR_CRYPTO,
            FfiError::VerifyFailed => BBS_ERROR_VERIFY_FAILED,
        }
    }
}

type FfiResult<T> = Result<T, FfiError>;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct BbsByteSlice {
    pub data: *const u8,
    pub len: usize,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct BbsByteBuffer {
    pub data: *mut u8,
    pub len: usize,
}

impl Default for BbsByteBuffer {
    fn default() -> Self {
        Self {
            data: ptr::null_mut(),
            len: 0,
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct BbsMessage {
    pub data: *const u8,
    pub len: usize,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct BbsIndexedMessage {
    pub index: u32,
    pub data: *const u8,
    pub len: usize,
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct BbsKeyPair {
    pub params: BbsByteBuffer,
    pub secret_key: BbsByteBuffer,
    pub public_key: BbsByteBuffer,
}

fn ffi_guard(op: impl FnOnce() -> FfiResult<()>) -> c_int {
    match catch_unwind(AssertUnwindSafe(op)) {
        Ok(Ok(())) => BBS_OK,
        Ok(Err(err)) => err.code(),
        Err(_) => BBS_ERROR_PANIC,
    }
}

unsafe fn read_bytes<'a>(data: *const u8, len: usize) -> FfiResult<&'a [u8]> {
    if len == 0 {
        return Ok(&[]);
    }
    if data.is_null() {
        return Err(FfiError::NullPointer);
    }
    Ok(slice::from_raw_parts(data, len))
}

unsafe fn read_slice<'a, T>(data: *const T, len: usize) -> FfiResult<&'a [T]> {
    if len == 0 {
        return Ok(&[]);
    }
    if data.is_null() {
        return Err(FfiError::NullPointer);
    }
    Ok(slice::from_raw_parts(data, len))
}

unsafe fn read_byte_slice<'a>(slice: BbsByteSlice) -> FfiResult<&'a [u8]> {
    read_bytes(slice.data, slice.len)
}

unsafe fn read_messages(data: *const BbsMessage, len: usize) -> FfiResult<Vec<Vec<u8>>> {
    if len == 0 {
        return Err(FfiError::InvalidLength);
    }
    read_slice(data, len)?
        .iter()
        .map(|msg| read_bytes(msg.data, msg.len).map(|bytes| bytes.to_vec()))
        .collect()
}

unsafe fn read_revealed_indices(
    data: *const u32,
    len: usize,
    message_count: usize,
) -> FfiResult<BTreeSet<usize>> {
    let mut indices = BTreeSet::new();
    for index in read_slice(data, len)? {
        let index = *index as usize;
        if index >= message_count || !indices.insert(index) {
            return Err(FfiError::InvalidIndex);
        }
    }
    Ok(indices)
}

unsafe fn read_revealed_messages(
    data: *const BbsIndexedMessage,
    len: usize,
) -> FfiResult<BTreeMap<usize, Fr>> {
    let mut messages = BTreeMap::new();
    for msg in read_slice(data, len)? {
        let index = msg.index as usize;
        if messages.contains_key(&index) {
            return Err(FfiError::InvalidIndex);
        }
        let bytes = read_bytes(msg.data, msg.len)?;
        messages.insert(index, hash_message(index, bytes));
    }
    Ok(messages)
}

fn buffer_from_vec(bytes: Vec<u8>) -> BbsByteBuffer {
    if bytes.is_empty() {
        return BbsByteBuffer::default();
    }

    let len = bytes.len();
    let boxed = bytes.into_boxed_slice();
    let data = Box::into_raw(boxed) as *mut u8;
    BbsByteBuffer { data, len }
}

unsafe fn free_buffer(buffer: BbsByteBuffer) {
    if buffer.data.is_null() || buffer.len == 0 {
        return;
    }

    let data = ptr::slice_from_raw_parts_mut(buffer.data, buffer.len);
    drop(Box::from_raw(data));
}

fn write_buffer(out: *mut BbsByteBuffer, bytes: Vec<u8>) -> FfiResult<()> {
    if out.is_null() {
        return Err(FfiError::NullPointer);
    }
    unsafe {
        *out = buffer_from_vec(bytes);
    }
    Ok(())
}

fn serialize_compressed<T: CanonicalSerialize>(value: &T) -> FfiResult<Vec<u8>> {
    let mut bytes = Vec::new();
    value
        .serialize_compressed(&mut bytes)
        .map_err(|_| FfiError::Serialize)?;
    Ok(bytes)
}

fn deserialize_compressed<T: CanonicalDeserialize>(bytes: &[u8]) -> FfiResult<T> {
    let mut reader = bytes;
    let value = T::deserialize_compressed(&mut reader).map_err(|_| FfiError::Deserialize)?;
    if !reader.is_empty() {
        return Err(FfiError::Deserialize);
    }
    Ok(value)
}

fn hash_message(index: usize, message: &[u8]) -> Fr {
    let mut domain = Vec::with_capacity(MESSAGE_DOMAIN_PREFIX.len() + 8);
    domain.extend_from_slice(MESSAGE_DOMAIN_PREFIX);
    domain.extend_from_slice(&(index as u64).to_be_bytes());
    hash_to_field::<Fr, Blake2b512>(&domain, message)
}

fn hash_messages(messages: &[Vec<u8>]) -> Vec<Fr> {
    messages
        .iter()
        .enumerate()
        .map(|(index, msg)| hash_message(index, msg))
        .collect()
}

fn proof_challenge_from_protocol(
    public_key: &PublicKey,
    protocol: &PoKOfSignatureG1Protocol<Bls12_381>,
    revealed_messages: &BTreeMap<usize, Fr>,
    params: &Params,
) -> FfiResult<Fr> {
    let mut bytes = Vec::new();
    public_key
        .serialize_compressed(&mut bytes)
        .map_err(|_| FfiError::Serialize)?;
    protocol
        .challenge_contribution(revealed_messages, params, &mut bytes)
        .map_err(|_| FfiError::Crypto)?;
    Ok(compute_random_oracle_challenge::<Fr, Blake2b512>(&bytes))
}

fn proof_challenge_from_proof(
    public_key: &PublicKey,
    proof: &Proof,
    revealed_messages: &BTreeMap<usize, Fr>,
    params: &Params,
) -> FfiResult<Fr> {
    let mut bytes = Vec::new();
    public_key
        .serialize_compressed(&mut bytes)
        .map_err(|_| FfiError::Serialize)?;
    proof
        .challenge_contribution(revealed_messages, params, &mut bytes)
        .map_err(|_| FfiError::Crypto)?;
    Ok(compute_random_oracle_challenge::<Fr, Blake2b512>(&bytes))
}

fn generate_keypair(message_count: u32) -> FfiResult<(Vec<u8>, Vec<u8>, Vec<u8>)> {
    if message_count == 0 {
        return Err(FfiError::InvalidLength);
    }

    let mut rng = OsRng;
    let params = Params::generate_using_rng(&mut rng, message_count);
    let keypair = KeypairG2::<Bls12_381>::generate_using_rng(&mut rng, &params);

    if !params.is_valid() || !keypair.public_key.is_valid() {
        return Err(FfiError::Crypto);
    }

    Ok((
        serialize_compressed(&params)?,
        serialize_compressed(&keypair.secret_key)?,
        serialize_compressed(&keypair.public_key)?,
    ))
}

fn sign(params: &Params, secret_key: &Secret, messages: &[Fr]) -> FfiResult<Signature> {
    let mut rng = OsRng;
    Signature::new(&mut rng, messages, secret_key, params).map_err(|_| FfiError::Crypto)
}

fn verify_signature(
    params: &Params,
    public_key: &PublicKey,
    messages: &[Fr],
    signature: &Signature,
) -> FfiResult<()> {
    signature
        .verify(messages, public_key.clone(), params.clone())
        .map_err(|_| FfiError::VerifyFailed)
}

fn create_proof(
    params: &Params,
    public_key: &PublicKey,
    signature: &Signature,
    messages: &[Fr],
    revealed_indices: &BTreeSet<usize>,
) -> FfiResult<Proof> {
    let mut rng = OsRng;
    let revealed_messages = revealed_indices
        .iter()
        .map(|index| (*index, messages[*index]))
        .collect::<BTreeMap<_, _>>();
    let messages_and_blindings = messages.iter().enumerate().map(|(index, msg)| {
        if revealed_indices.contains(&index) {
            MessageOrBlinding::RevealMessage(msg)
        } else {
            MessageOrBlinding::BlindMessageRandomly(msg)
        }
    });

    let protocol =
        PoKOfSignatureG1Protocol::init(&mut rng, signature, params, messages_and_blindings)
            .map_err(|_| FfiError::Crypto)?;
    let challenge =
        proof_challenge_from_protocol(public_key, &protocol, &revealed_messages, params)?;
    protocol.gen_proof(&challenge).map_err(|_| FfiError::Crypto)
}

fn verify_proof(
    params: &Params,
    public_key: &PublicKey,
    proof: &Proof,
    revealed_messages: &BTreeMap<usize, Fr>,
) -> FfiResult<()> {
    let challenge = proof_challenge_from_proof(public_key, proof, revealed_messages, params)?;
    proof
        .verify(
            revealed_messages,
            &challenge,
            public_key.clone(),
            params.clone(),
        )
        .map_err(|_| FfiError::VerifyFailed)
}

/// Frees a byte buffer allocated by this library.
///
/// # Safety
/// The buffer must have been returned by this library and must not be freed more than once.
#[no_mangle]
pub unsafe extern "C" fn bbs_free_buffer(buffer: BbsByteBuffer) {
    free_buffer(buffer);
}

/// Frees all buffers held by a keypair allocated by `bbs_generate_keypair`.
///
/// # Safety
/// `keypair` must be null or point to a valid `BbsKeyPair`. After this call, all embedded buffers
/// are reset to null/zero.
#[no_mangle]
pub unsafe extern "C" fn bbs_free_keypair(keypair: *mut BbsKeyPair) {
    if keypair.is_null() {
        return;
    }
    let keypair = &mut *keypair;
    free_buffer(keypair.params);
    free_buffer(keypair.secret_key);
    free_buffer(keypair.public_key);
    *keypair = BbsKeyPair::default();
}

/// Returns a static, null-terminated description for a status code.
#[no_mangle]
pub extern "C" fn bbs_status_message(status: c_int) -> *const c_char {
    match status {
        BBS_OK => b"ok\0".as_ptr(),
        BBS_ERROR_NULL_POINTER => b"null pointer\0".as_ptr(),
        BBS_ERROR_INVALID_LENGTH => b"invalid length\0".as_ptr(),
        BBS_ERROR_INVALID_INDEX => b"invalid index\0".as_ptr(),
        BBS_ERROR_DESERIALIZE => b"deserialize error\0".as_ptr(),
        BBS_ERROR_SERIALIZE => b"serialize error\0".as_ptr(),
        BBS_ERROR_CRYPTO => b"cryptographic operation failed\0".as_ptr(),
        BBS_ERROR_VERIFY_FAILED => b"verification failed\0".as_ptr(),
        BBS_ERROR_PANIC => b"panic\0".as_ptr(),
        _ => b"unknown status\0".as_ptr(),
    }
    .cast()
}

/// Generates signature parameters and a BBS+ keypair for `message_count` messages.
///
/// The returned params, secret key, and public key are Dock canonical compressed bytes. The caller
/// owns the returned buffers and must free them with `bbs_free_keypair`.
///
/// # Safety
/// `out_keypair` must point to writable memory for one `BbsKeyPair`.
#[no_mangle]
pub unsafe extern "C" fn bbs_generate_keypair(
    message_count: u32,
    out_keypair: *mut BbsKeyPair,
) -> c_int {
    ffi_guard(|| {
        if out_keypair.is_null() {
            return Err(FfiError::NullPointer);
        }

        let (params, secret_key, public_key) = generate_keypair(message_count)?;
        *out_keypair = BbsKeyPair {
            params: buffer_from_vec(params),
            secret_key: buffer_from_vec(secret_key),
            public_key: buffer_from_vec(public_key),
        };
        Ok(())
    })
}

/// Signs raw byte messages with Dock BBS+.
///
/// Messages are hashed to BLS12-381 field elements internally using a stable domain based on their
/// zero-based position. The returned signature is Dock canonical compressed bytes and must be freed
/// with `bbs_free_buffer`.
///
/// # Safety
/// All input slices must be valid for their lengths. `out_signature` must point to writable memory
/// for one `BbsByteBuffer`.
#[no_mangle]
pub unsafe extern "C" fn bbs_sign(
    params: BbsByteSlice,
    secret_key: BbsByteSlice,
    messages: *const BbsMessage,
    message_count: usize,
    out_signature: *mut BbsByteBuffer,
) -> c_int {
    ffi_guard(|| {
        let params = deserialize_compressed::<Params>(read_byte_slice(params)?)?;
        let secret_key = deserialize_compressed::<Secret>(read_byte_slice(secret_key)?)?;
        let raw_messages = read_messages(messages, message_count)?;
        let messages = hash_messages(&raw_messages);
        let signature = sign(&params, &secret_key, &messages)?;
        write_buffer(out_signature, serialize_compressed(&signature)?)
    })
}

/// Verifies a Dock BBS+ signature over raw byte messages.
///
/// The verifier hashes messages the same way as `bbs_sign`. Returns `BBS_OK` only when the
/// signature verifies; invalid signatures return `BBS_ERROR_VERIFY_FAILED`.
///
/// # Safety
/// All input slices and the `messages` array must be valid for their lengths.
#[no_mangle]
pub unsafe extern "C" fn bbs_verify_signature(
    params: BbsByteSlice,
    public_key: BbsByteSlice,
    messages: *const BbsMessage,
    message_count: usize,
    signature: BbsByteSlice,
) -> c_int {
    ffi_guard(|| {
        let params = deserialize_compressed::<Params>(read_byte_slice(params)?)?;
        let public_key = deserialize_compressed::<PublicKey>(read_byte_slice(public_key)?)?;
        let signature = deserialize_compressed::<Signature>(read_byte_slice(signature)?)?;
        let raw_messages = read_messages(messages, message_count)?;
        let messages = hash_messages(&raw_messages);
        verify_signature(&params, &public_key, &messages, &signature)
    })
}

/// Creates a proof of knowledge of a BBS+ signature with selective disclosure.
///
/// `revealed_indices` contains zero-based message indexes to disclose. Non-revealed messages are
/// hidden in the proof. The returned proof is Dock canonical compressed bytes and must be freed with
/// `bbs_free_buffer`.
///
/// # Safety
/// All input slices and arrays must be valid for their lengths. `out_proof` must point to writable
/// memory for one `BbsByteBuffer`.
#[no_mangle]
pub unsafe extern "C" fn bbs_create_proof(
    params: BbsByteSlice,
    public_key: BbsByteSlice,
    signature: BbsByteSlice,
    messages: *const BbsMessage,
    message_count: usize,
    revealed_indices: *const u32,
    revealed_indices_count: usize,
    out_proof: *mut BbsByteBuffer,
) -> c_int {
    ffi_guard(|| {
        let params = deserialize_compressed::<Params>(read_byte_slice(params)?)?;
        let public_key = deserialize_compressed::<PublicKey>(read_byte_slice(public_key)?)?;
        let signature = deserialize_compressed::<Signature>(read_byte_slice(signature)?)?;
        let raw_messages = read_messages(messages, message_count)?;
        let messages = hash_messages(&raw_messages);
        let revealed_indices =
            read_revealed_indices(revealed_indices, revealed_indices_count, messages.len())?;
        let proof = create_proof(
            &params,
            &public_key,
            &signature,
            &messages,
            &revealed_indices,
        )?;
        write_buffer(out_proof, serialize_compressed(&proof)?)
    })
}

/// Verifies a selective-disclosure BBS+ proof.
///
/// `revealed_messages` must contain the same zero-based indexes and raw message bytes disclosed by
/// the prover. Returns `BBS_OK` only when the proof verifies.
///
/// # Safety
/// All input slices and the `revealed_messages` array must be valid for their lengths.
#[no_mangle]
pub unsafe extern "C" fn bbs_verify_proof(
    params: BbsByteSlice,
    public_key: BbsByteSlice,
    proof: BbsByteSlice,
    revealed_messages: *const BbsIndexedMessage,
    revealed_message_count: usize,
) -> c_int {
    ffi_guard(|| {
        let params = deserialize_compressed::<Params>(read_byte_slice(params)?)?;
        let public_key = deserialize_compressed::<PublicKey>(read_byte_slice(public_key)?)?;
        let proof = deserialize_compressed::<Proof>(read_byte_slice(proof)?)?;
        let revealed_messages = read_revealed_messages(revealed_messages, revealed_message_count)?;
        verify_proof(&params, &public_key, &proof, &revealed_messages)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn byte_slice(bytes: &[u8]) -> BbsByteSlice {
        BbsByteSlice {
            data: bytes.as_ptr(),
            len: bytes.len(),
        }
    }

    unsafe fn buffer_slice(buffer: BbsByteBuffer) -> BbsByteSlice {
        BbsByteSlice {
            data: buffer.data.cast_const(),
            len: buffer.len,
        }
    }

    unsafe fn copy_buffer(buffer: BbsByteBuffer) -> Vec<u8> {
        slice::from_raw_parts(buffer.data, buffer.len).to_vec()
    }

    fn ffi_message(message: &[u8]) -> BbsMessage {
        BbsMessage {
            data: message.as_ptr(),
            len: message.len(),
        }
    }

    fn ffi_indexed_message(index: u32, message: &[u8]) -> BbsIndexedMessage {
        BbsIndexedMessage {
            index,
            data: message.as_ptr(),
            len: message.len(),
        }
    }

    #[test]
    fn sign_and_verify_round_trip() {
        unsafe {
            let mut keypair = BbsKeyPair::default();
            assert_eq!(bbs_generate_keypair(3, &mut keypair), BBS_OK);

            let messages = [
                b"alice".as_slice(),
                b"US".as_slice(),
                b"1716328800".as_slice(),
            ];
            let ffi_messages = messages.map(ffi_message);

            let mut signature = BbsByteBuffer::default();
            assert_eq!(
                bbs_sign(
                    buffer_slice(keypair.params),
                    buffer_slice(keypair.secret_key),
                    ffi_messages.as_ptr(),
                    ffi_messages.len(),
                    &mut signature,
                ),
                BBS_OK
            );

            assert_eq!(
                bbs_verify_signature(
                    buffer_slice(keypair.params),
                    buffer_slice(keypair.public_key),
                    ffi_messages.as_ptr(),
                    ffi_messages.len(),
                    buffer_slice(signature),
                ),
                BBS_OK
            );

            let bad_messages = [
                ffi_message(b"alice"),
                ffi_message(b"CA"),
                ffi_message(b"1716328800"),
            ];
            assert_eq!(
                bbs_verify_signature(
                    buffer_slice(keypair.params),
                    buffer_slice(keypair.public_key),
                    bad_messages.as_ptr(),
                    bad_messages.len(),
                    buffer_slice(signature),
                ),
                BBS_ERROR_VERIFY_FAILED
            );

            bbs_free_buffer(signature);
            bbs_free_keypair(&mut keypair);
        }
    }

    #[test]
    fn create_and_verify_proof_round_trip() {
        unsafe {
            let mut keypair = BbsKeyPair::default();
            assert_eq!(bbs_generate_keypair(3, &mut keypair), BBS_OK);

            let messages = [
                b"alice".as_slice(),
                b"US".as_slice(),
                b"1716328800".as_slice(),
            ];
            let ffi_messages = messages.map(ffi_message);

            let mut signature = BbsByteBuffer::default();
            assert_eq!(
                bbs_sign(
                    buffer_slice(keypair.params),
                    buffer_slice(keypair.secret_key),
                    ffi_messages.as_ptr(),
                    ffi_messages.len(),
                    &mut signature,
                ),
                BBS_OK
            );

            let revealed = [0_u32, 2_u32];
            let mut proof = BbsByteBuffer::default();
            assert_eq!(
                bbs_create_proof(
                    buffer_slice(keypair.params),
                    buffer_slice(keypair.public_key),
                    buffer_slice(signature),
                    ffi_messages.as_ptr(),
                    ffi_messages.len(),
                    revealed.as_ptr(),
                    revealed.len(),
                    &mut proof,
                ),
                BBS_OK
            );

            let revealed_messages = [
                ffi_indexed_message(0, b"alice"),
                ffi_indexed_message(2, b"1716328800"),
            ];
            assert_eq!(
                bbs_verify_proof(
                    buffer_slice(keypair.params),
                    buffer_slice(keypair.public_key),
                    buffer_slice(proof),
                    revealed_messages.as_ptr(),
                    revealed_messages.len(),
                ),
                BBS_OK
            );

            let tampered_revealed_messages = [
                ffi_indexed_message(0, b"alice"),
                ffi_indexed_message(2, b"1716328801"),
            ];
            assert_eq!(
                bbs_verify_proof(
                    buffer_slice(keypair.params),
                    buffer_slice(keypair.public_key),
                    buffer_slice(proof),
                    tampered_revealed_messages.as_ptr(),
                    tampered_revealed_messages.len(),
                ),
                BBS_ERROR_VERIFY_FAILED
            );

            bbs_free_buffer(proof);
            bbs_free_buffer(signature);
            bbs_free_keypair(&mut keypair);
        }
    }

    #[test]
    fn rejects_tampered_signature_bytes() {
        unsafe {
            let mut keypair = BbsKeyPair::default();
            assert_eq!(bbs_generate_keypair(1, &mut keypair), BBS_OK);

            let message = [ffi_message(b"alice")];
            let mut signature = BbsByteBuffer::default();
            assert_eq!(
                bbs_sign(
                    buffer_slice(keypair.params),
                    buffer_slice(keypair.secret_key),
                    message.as_ptr(),
                    message.len(),
                    &mut signature,
                ),
                BBS_OK
            );

            let mut tampered = copy_buffer(signature);
            let last = tampered.len() - 1;
            tampered[last] ^= 1;

            assert_ne!(
                bbs_verify_signature(
                    buffer_slice(keypair.params),
                    buffer_slice(keypair.public_key),
                    message.as_ptr(),
                    message.len(),
                    byte_slice(&tampered),
                ),
                BBS_OK
            );

            bbs_free_buffer(signature);
            bbs_free_keypair(&mut keypair);
        }
    }
}
