use std::fmt;

#[derive(Debug, PartialEq)]
pub enum Error {
	NoSpec,
	InvalidSpec,
	// TODO: NoSchema bypass the return of an Option in
	// schema::Schema::from_path, it's not an actual error
	NoSchema,
	InvalidSchema,
}

impl fmt::Display for Error {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		use Error::*;

		match self {
			NoSpec => write!(f, "Missing package file"),
			InvalidSpec => write!(f, "Invalid package file"),
			NoSchema => write!(f, "No schema file to validate your values"),
			InvalidSchema => write!(f, "Invalid schema file"),
		}
	}
}

pub type Result<T> = std::result::Result<T, Error>;
