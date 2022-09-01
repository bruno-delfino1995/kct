use thiserror::Error as TError;

#[derive(TError, Debug, PartialEq, Eq)]
pub enum Error {
	#[error("Failed to parse input: {0}")]
	InvalidInput(String),
}
