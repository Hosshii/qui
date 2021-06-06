use std::error::Error as StdError;
use std::fmt::{self, Debug, Display};

use rust_traq::apis;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum MyError {
    #[error("{0}")]
    ApiError(String),
}

impl<T: Debug> From<apis::Error<T>> for MyError {
    fn from(from: apis::Error<T>) -> Self {
        Self::ApiError(format!("{:?}", from))
    }
}
