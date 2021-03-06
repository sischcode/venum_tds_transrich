use strum_macros::Display;
use thiserror::Error;

use venum::{errors::VenumError, venum::Value};
use venum_tds::errors::VenumTdsError;

#[derive(Debug, PartialEq, Display, Clone)]
pub enum WrappedErrors {
    VenumError(VenumError),
    VenumTdsError(VenumTdsError),
}

#[derive(Debug, PartialEq, Display, Clone)]
pub enum ContainerOpsErrors {
    Generic { msg: String },
    DivideItemError { idx: usize, msg: String },
}

#[derive(Debug, PartialEq, Display, Clone)]
pub enum VenumTdsTransRichError {
    Generic { msg: String },
    Wrapped(WrappedErrors),
    Split(SplitError),
    ContainerOps(ContainerOpsErrors),
}

#[derive(Error, Debug, PartialEq, Clone)]
#[error("error: {msg:?}; problem value: {src_val:?}. Details: {details:?}")]
pub struct SplitError {
    msg: String,
    src_val: Option<Value>,
    details: Option<String>,
}

impl SplitError {
    pub fn minim(msg: String) -> Self {
        Self {
            msg,
            src_val: None,
            details: None,
        }
    }
    pub fn from(msg: String, src_val: Option<Value>, details: Option<String>) -> Self {
        Self {
            msg,
            src_val,
            details,
        }
    }
}

pub type Result<T> = std::result::Result<T, VenumTdsTransRichError>;

impl From<VenumTdsError> for VenumTdsTransRichError {
    fn from(ve: VenumTdsError) -> Self {
        VenumTdsTransRichError::Wrapped(WrappedErrors::VenumTdsError(ve))
    }
}

impl From<VenumError> for VenumTdsTransRichError {
    fn from(ve: VenumError) -> Self {
        VenumTdsTransRichError::Wrapped(WrappedErrors::VenumError(ve))
    }
}
