//! Errors related to the Substrate types decoding/conversion.
use hex::FromHexError;

#[derive(thiserror::Error, Clone, Debug)]
pub enum DecodeError {
    #[error("Decode error: {0}")]
    Error(String),
}

impl From<FromHexError> for DecodeError {
    fn from(error: FromHexError) -> Self {
        Self::Error(error.to_string())
    }
}

impl From<parity_scale_codec::Error> for DecodeError {
    fn from(error: parity_scale_codec::Error) -> Self {
        Self::Error(error.to_string())
    }
}
