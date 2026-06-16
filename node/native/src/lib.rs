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

#[napi(js_name = "verifyProof")]
pub fn verify_proof(
    params: Buffer,
    public_key: Buffer,
    proof: Buffer,
    revealed_messages: Vec<RevealedMessage>,
) -> Result<bool> {
    let revealed_messages = revealed_messages
        .iter()
        .map(|message| (message.index, message.data.as_ref()))
        .collect::<Vec<_>>();

    match bbsplus_core::verify_proof_bytes(
        params.as_ref(),
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
