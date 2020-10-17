use std::fmt;

#[derive(Debug, PartialEq)]
pub enum Error {
	NoMain,
	InvalidFormat,
	NoSpec,
	InvalidSpec,
	NoSchema,
	InvalidSchema,
	NoValues,
	InvalidValues,
	RenderIssue(String),
	InvalidOutput,
}

impl fmt::Display for Error {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		use Error::*;

		match self {
			NoMain => write!(f, "Package doesn't have a main template"),
			InvalidFormat => write!(f, "Package is neither directory nor a .tgz"),
			NoSpec => write!(f, "Missing package file"),
			InvalidSpec => write!(f, "Invalid package file"),
			NoSchema => write!(f, "No schema file to validate your values"),
			InvalidSchema => write!(f, "Invalid schema file"),
			NoValues => write!(f, "No values where provided"),
			InvalidValues => write!(f, "The values provided don't match the schema"),
			RenderIssue(err) => write!(
				f,
				"An error happened while rendering your templates: {}",
				err
			),
			InvalidOutput => write!(f, "Your template couldn't be parsed as JSON"),
		}
	}
}

pub type Result<T> = std::result::Result<T, Error>;
