use thiserror::Error;

#[derive(Error, Debug, PartialEq, Eq)]
pub enum Error {
	#[error("No validation provided")]
	NoValidator,
	#[error("No input was provided")]
	NoInput,
	#[error("The input provided doesn't match the schema: {0}")]
	InvalidInput(String),
	#[error("An error happened while rendering your templates: {0}")]
	RenderIssue(String),
	#[error("Your template couldn't be parsed as JSON")]
	InvalidOutput,
}

pub type Result<T> = std::result::Result<T, Error>;
