use std::ffi::{c_char, CStr};
use std::os::raw::c_int;

use crate::{
    ffi_guard, read_byte_slice, read_slice, write_buffer, BbsByteBuffer, BbsByteSlice, FfiError,
    FfiResult,
};

/// Error returned by nationality canonicalization helpers.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum UtilsError {
    InvalidLength,
}

impl UtilsError {
    fn into_ffi(self) -> FfiError {
        match self {
            UtilsError::InvalidLength => FfiError::InvalidLength,
        }
    }
}

/// Trims leading and trailing whitespace, then uppercases the string.
pub fn canonical_string(value: &str) -> String {
    value.trim().to_uppercase()
}

/// Canonicalizes a nationality code to exactly two ASCII letters `A`–`Z`.
pub fn canonical_nationality(value: &str) -> Result<String, UtilsError> {
    let canonical = canonical_string(value);
    let bytes = canonical.as_bytes();
    if bytes.len() == 2 && bytes[0].is_ascii_uppercase() && bytes[1].is_ascii_uppercase() {
        Ok(canonical)
    } else {
        Err(UtilsError::InvalidLength)
    }
}

/// Canonicalizes nationality codes, sorts them alphabetically, and concatenates.
pub fn canonical_nationality_list(values: &[&str]) -> Result<String, UtilsError> {
    let mut items = values
        .iter()
        .map(|value| canonical_nationality(value))
        .collect::<Result<Vec<_>, _>>()?;
    items.sort_unstable();
    Ok(items.concat())
}

unsafe fn utf8_from_byte_slice<'a>(input: BbsByteSlice) -> FfiResult<&'a str> {
    let bytes = read_byte_slice(input)?;
    std::str::from_utf8(bytes).map_err(|_| FfiError::Deserialize)
}

/// Canonicalizes a UTF-8 string (trim + uppercase).
///
/// # Safety
/// `out` must be null or point to a valid `BbsByteBuffer`. On success the buffer is owned by the
/// caller and must be freed with `bbs_free_buffer`.
#[no_mangle]
pub unsafe extern "C" fn bbs_canonical_string(
    input: BbsByteSlice,
    out: *mut BbsByteBuffer,
) -> c_int {
    ffi_guard(|| {
        let value = utf8_from_byte_slice(input)?;
        write_buffer(out, canonical_string(value).into_bytes())
    })
}

/// Canonicalizes a nationality code to two ASCII letters `A`–`Z`.
///
/// # Safety
/// `out` must be null or point to a valid `BbsByteBuffer`. On success the buffer is owned by the
/// caller and must be freed with `bbs_free_buffer`.
#[no_mangle]
pub unsafe extern "C" fn bbs_canonical_nationality(
    input: BbsByteSlice,
    out: *mut BbsByteBuffer,
) -> c_int {
    ffi_guard(|| {
        let value = utf8_from_byte_slice(input)?;
        let canonical = canonical_nationality(value).map_err(UtilsError::into_ffi)?;
        write_buffer(out, canonical.into_bytes())
    })
}

/// Canonicalizes nationality codes, sorts them, and concatenates the result.
///
/// # Safety
/// `items` must be null when `item_count` is 0, or point to `item_count` null-terminated UTF-8
/// C strings. `out` must be null or point to a valid `BbsByteBuffer`. On success the buffer is
/// owned by the caller and must be freed with `bbs_free_buffer`.
#[no_mangle]
pub unsafe extern "C" fn bbs_canonical_nationality_list(
    items: *const *const c_char,
    item_count: usize,
    out: *mut BbsByteBuffer,
) -> c_int {
    ffi_guard(|| {
        let pointers = read_slice(items, item_count)?;
        let mut values = Vec::with_capacity(pointers.len());
        for &pointer in pointers {
            if pointer.is_null() {
                return Err(FfiError::NullPointer);
            }
            let value = CStr::from_ptr(pointer)
                .to_str()
                .map_err(|_| FfiError::Deserialize)?;
            values.push(value);
        }
        let canonical = canonical_nationality_list(&values).map_err(UtilsError::into_ffi)?;
        write_buffer(out, canonical.into_bytes())
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canonical_string_trims_and_uppercases() {
        assert_eq!(canonical_string("  cz "), "CZ");
        assert_eq!(canonical_string("pl"), "PL");
        assert_eq!(canonical_string(""), "");
    }

    #[test]
    fn canonical_nationality_accepts_ascii_alpha2() {
        assert_eq!(canonical_nationality("cz ").unwrap(), "CZ");
        assert_eq!(canonical_nationality("IT").unwrap(), "IT");
        assert_eq!(canonical_nationality("  us").unwrap(), "US");
    }

    #[test]
    fn canonical_nationality_rejects_invalid_codes() {
        assert_eq!(canonical_nationality(""), Err(UtilsError::InvalidLength));
        assert_eq!(canonical_nationality("abc"), Err(UtilsError::InvalidLength));
        assert_eq!(canonical_nationality("A1"), Err(UtilsError::InvalidLength));
        assert_eq!(canonical_nationality("ÇZ"), Err(UtilsError::InvalidLength));
        assert_eq!(canonical_nationality("A-"), Err(UtilsError::InvalidLength));
        assert_eq!(canonical_nationality("A B"), Err(UtilsError::InvalidLength));
    }

    #[test]
    fn canonical_nationality_list_sorts_and_concats() {
        assert_eq!(
            canonical_nationality_list(&["it", "pl", "cz "]).unwrap(),
            "CZITPL"
        );
        assert_eq!(canonical_nationality_list(&[]).unwrap(), "");
    }

    #[test]
    fn canonical_nationality_list_rejects_empty_entry() {
        assert_eq!(
            canonical_nationality_list(&["", "us"]),
            Err(UtilsError::InvalidLength)
        );
    }
}
