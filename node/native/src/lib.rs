use napi::{bindgen_prelude::Buffer, Error, Result, Status};
use napi_derive::napi;

#[napi(object)]
pub struct RevealedMessage {
    pub index: u32,
    pub data: Buffer,
}

#[napi(js_name = "pidOrder")]
pub fn pid_order() -> Vec<String> {
    bbsplus_core::pid_order()
        .iter()
        .map(|item| (*item).to_owned())
        .collect()
}

#[napi(js_name = "canonicalString")]
pub fn canonical_string(value: String) -> String {
    bbsplus_core::canonical_string(&value)
}

#[napi(js_name = "canonicalNationality")]
pub fn canonical_nationality(value: String) -> Result<String> {
    bbsplus_core::canonical_nationality(&value).map_err(|_| {
        Error::new(
            Status::GenericFailure,
            format!(
                "libbbsplus: {}",
                bbsplus_core::status_message(bbsplus_core::BBS_ERROR_INVALID_LENGTH)
            ),
        )
    })
}

#[napi(js_name = "canonicalNationalityList")]
pub fn canonical_nationality_list(values: Vec<String>) -> Result<String> {
    let refs = values.iter().map(String::as_str).collect::<Vec<_>>();
    bbsplus_core::canonical_nationality_list(&refs).map_err(|_| {
        Error::new(
            Status::GenericFailure,
            format!(
                "libbbsplus: {}",
                bbsplus_core::status_message(bbsplus_core::BBS_ERROR_INVALID_LENGTH)
            ),
        )
    })
}

#[napi(js_name = "verifyProof")]
pub fn verify_proof(
    public_key: Buffer,
    proof: Buffer,
    revealed_messages: Vec<RevealedMessage>,
) -> Result<bool> {
    let revealed_messages = revealed_messages
        .iter()
        .map(|message| (message.index, message.data.as_ref()))
        .collect::<Vec<_>>();

    match bbsplus_core::verify_proof_bytes(
        public_key.as_ref(),
        proof.as_ref(),
        &revealed_messages,
    ) {
        bbsplus_core::BBS_OK => Ok(true),
        bbsplus_core::BBS_ERROR_VERIFY_FAILED => Ok(false),
        status => Err(Error::new(
            Status::GenericFailure,
            format!("libbbsplus: {}", bbsplus_core::status_message(status)),
        )),
    }
}
