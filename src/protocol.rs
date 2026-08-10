use std::ffi::c_char;

/// Maximum number of messages supported by this wrapper's protocol params.
pub const MAX_MESSAGE_COUNT: u32 = 20;

/// PID identifiers as defined in
/// https://eudi.dev/2.4.0/annexes/annex-3/annex-3.01-pid-rulebook/
pub const PID_ORDER: [&str; 6] = [
    "family_name",
    "given_name",
    "birth_date",
    "birth_place",
    "nationality",
    // Used to verify whether someone's nationality appears in their nationalities list.
    "derived_nationality",
];

#[repr(C)]
#[derive(Clone, Copy)]
pub struct BbsStringArray {
    pub data: *const *const c_char,
    pub len: usize,
}

#[repr(transparent)]
struct StaticStrPtr(*const c_char);

unsafe impl Sync for StaticStrPtr {}

const PID_FAMILY_NAME: &[u8] = b"family_name\0";
const PID_GIVEN_NAME: &[u8] = b"given_name\0";
const PID_BIRTH_DATE: &[u8] = b"birth_date\0";
const PID_BIRTH_PLACE: &[u8] = b"birth_place\0";
const PID_NATIONALITY: &[u8] = b"nationality\0";
const PID_DERIVED_NATIONALITY: &[u8] = b"derived_nationality\0";

static PID_ORDER_C: [StaticStrPtr; PID_ORDER.len()] = [
    StaticStrPtr(PID_FAMILY_NAME.as_ptr().cast()),
    StaticStrPtr(PID_GIVEN_NAME.as_ptr().cast()),
    StaticStrPtr(PID_BIRTH_DATE.as_ptr().cast()),
    StaticStrPtr(PID_BIRTH_PLACE.as_ptr().cast()),
    StaticStrPtr(PID_NATIONALITY.as_ptr().cast()),
    StaticStrPtr(PID_DERIVED_NATIONALITY.as_ptr().cast()),
];

/// Returns the ordered PID identifiers supported by this wrapper.
///
/// The returned strings are static, null-terminated, and owned by the library.
#[no_mangle]
pub extern "C" fn bbs_pid_order() -> BbsStringArray {
    BbsStringArray {
        data: PID_ORDER_C.as_ptr().cast(),
        len: PID_ORDER_C.len(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{ffi::CStr, slice};

    #[test]
    fn exposes_pid_order() {
        let raw = bbs_pid_order();
        let items = unsafe { slice::from_raw_parts(raw.data, raw.len) };
        let values = items
            .iter()
            .map(|item| unsafe { CStr::from_ptr(*item).to_str().unwrap() })
            .collect::<Vec<_>>();

        assert_eq!(values, PID_ORDER);
    }
}
