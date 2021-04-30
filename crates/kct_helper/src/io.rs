use std::fs;
use std::io::{self, Read};
use std::path::Path;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum Error {
	#[error("Path is not a file")]
	NotFile,
	#[error("File not found")]
	NotFound,
	#[error("Unable to read")]
	UnableToRead,
}

pub fn from_file(file: &Path) -> Result<String, Error> {
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
