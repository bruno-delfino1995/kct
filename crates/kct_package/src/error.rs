use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum Error {
	#[error("Package doesn't have a main template")]
	NoMain,
	#[error("Package is neither directory nor a .tgz")]
	InvalidFormat,
	#[error("Missing package file")]
	NoSpec,
	#[error("Invalid package file")]
	InvalidSpec,
	#[error("No schema file to validate your input")]
	NoSchema,
	#[error("Invalid schema file")]
	InvalidSchema,
	#[error("No input was provided")]
	NoInput,
	#[error("The input provided doesn't match the schema")]
	InvalidInput,
	#[error("An error happened while rendering your templates: {0}")]
	RenderIssue(String),
	#[error("Your template couldn't be parsed as JSON")]
	InvalidOutput,
}

pub type Result<T> = std::result::Result<T, Error>;
