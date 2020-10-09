use std::fmt;
use std::fs;
use std::io::{self, Read};
use std::path::PathBuf;

pub enum Error {
	NotFile,
	NotFound,
	UnableToRead,
}

impl fmt::Display for Error {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		use Error::*;

		match self {
			NotFile => write!(f, "Path is not a file"),
			NotFound => write!(f, "File not found"),
			UnableToRead => write!(f, "Unable to read"),
		}
	}
}

pub fn from_file(file: &PathBuf) -> Result<String, Error> {
	if !file.exists() {
		return Err(Error::NotFound);
	}

	if !file.is_file() {
		return Err(Error::NotFile);
	}

	let contents = fs::read_to_string(file).map_err(|_err| Error::UnableToRead)?;

	Ok(contents)
}

pub fn from_stdin() -> Result<String, Error> {
	let mut contents = String::new();
	io::stdin()
		.read_to_string(&mut contents)
		.map_err(|_err| Error::UnableToRead)?;

	Ok(contents)
}
