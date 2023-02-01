use kct_jsonnet::Error as JsonnetError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
	#[error("No target to compile")]
	NoTarget,
	#[error("No validation provided")]
	NoValidator,
	#[error("No input was provided")]
	NoInput,
	#[error("The input provided is invalid: {0}")]
	InvalidInput(String),
	#[error("Your template couldn't be parsed as JSON")]
	InvalidOutput,
	#[error("Your context is invalid")]
	Context(#[from] Context),
	#[error(transparent)]
	Executable(#[from] JsonnetError),
}

#[derive(Error, Debug)]
pub enum Context {
	#[error("root is required")]
	NoRoot,
}
