use std::fs;
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum Error {
	#[error("Path is not a file")]
	NotFile,
	#[error("Path is not a directory")]
	NotDirectory,
	#[error("File not found")]
	NotFound,
	#[error("Unable to read")]
	UnableToRead,
	#[error("Unable to write")]
	UnableToWrite,
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

pub fn ensure_dir_exists(base: &Path, dir: &Path) -> Result<PathBuf, Error> {
	let dir = if dir.is_absolute() {
		dir.to_path_buf()
	} else {
		let mut target = base.to_path_buf();
		target.push(dir);

		target
	};

	if dir.is_file() {
		Err(Error::NotDirectory)
	} else if dir.is_dir() {
		Ok(dir)
	} else {
		fs::create_dir(dir.clone()).map_err(|_err| Error::UnableToWrite)?;

		Ok(dir)
	}
}

pub fn write_contents(path: &Path, contents: &str) -> Result<(), Error> {
	let parent = match path.parent() {
		None => return Err(Error::NotDirectory),
		Some(parent) => parent,
	};

	fs::create_dir_all(parent).map_err(|_err| Error::UnableToWrite)?;
	fs::write(path, contents).map_err(|_err| Error::UnableToWrite)
}
