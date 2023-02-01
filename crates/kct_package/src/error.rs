use kct_compiler::Error as CompilerError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
	#[error("Package doesn't have a main template")]
	NoMain,
	#[error("Package is a directory")]
	InvalidFormat,
	#[error("Missing package file")]
	NoSpec,
	#[error("Invalid package file")]
	InvalidSpec,
	#[error("No schema file to validate your example")]
	NoSchema,
	#[error("Invalid schema file")]
	InvalidSchema,
	#[error("No example file")]
	NoExample,
	#[error("Invalid example file")]
	InvalidExample,
	#[error(transparent)]
	Compilation(#[from] CompilerError),
}
