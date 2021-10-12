#[derive(thiserror::Error, Clone, Debug)]
pub enum DecodeError {
    #[error("Decode error: {0}")]
    Error(String),
}

impl From<parity_scale_codec::Error> for DecodeError {
    fn from(error: parity_scale_codec::Error) -> Self {
        Self::Error(error.to_string())
    }
}

impl From<crate::substrate::argument::ArgumentDecodeError> for DecodeError {
    fn from(error: crate::substrate::argument::ArgumentDecodeError) -> Self {
        Self::Error(error.to_string())
    }
}
