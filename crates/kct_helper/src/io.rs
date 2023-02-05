use std::env;
use std::fs;
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use std::str::FromStr;

use anyhow::Result;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
	#[error("Path is not a file")]
	NotFile,
	#[error("Path is not a directory")]
	NotDirectory,
	#[error("Path not found")]
	NotFound(PathBuf),
	#[error("Unable to read")]
	UnableToRead,
	#[error("Unable to write")]
	UnableToWrite,
	#[error("Unable to get current working directory")]
	NoCwd,
}

#[derive(Clone)]
pub enum Location {
	Standard,
	Path(PathBuf),
}

impl FromStr for Location {
	type Err = Error;

	fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
		if s == "-" {
			Ok(Self::Standard)
		} else {
			let base = env::current_dir().map_err(|_| Error::NoCwd)?;

			let path = PathBuf::from(s);
			if exists(&base, &path) {
				Ok(Self::Path(path))
			} else {
				Err(Error::NotFound(path))
			}
		}
	}
}

impl Location {
	pub fn read(self) -> Result<String, Error> {
		match self {
			Self::Standard => from_stdin().map_err(|_| Error::UnableToRead),
			Self::Path(path) => from_file(&path).map_err(|_| Error::UnableToRead),
		}
	}

	pub fn write(self, documents: Vec<(PathBuf, String)>) -> Result<String, Error> {
		match self {
			Self::Standard => {
				let contents = documents
					.into_iter()
					.map(|(_path, object)| object)
					.collect();

				Ok(contents)
			}
			Self::Path(root) => {
				for (path, contents) in documents {
					let target = {
						let mut base = root.to_path_buf();

						base.push(path.strip_prefix("/").unwrap());
						base.with_extension("yaml")
					};

					write_contents(&target, &contents)?;
				}

				Ok(format!("{}", root.display()))
			}
		}
	}
}

pub fn from_file(file: &Path) -> Result<String, Error> {
	if !file.exists() {
		return Err(Error::NotFound(file.to_path_buf()));
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

pub fn exists(base: &Path, path: &Path) -> bool {
	let path = if path.is_absolute() {
		path.to_path_buf()
	} else {
		let mut target = base.to_path_buf();
		target.push(path);

		target
	};

	path.exists()
}

pub fn ensure_dir_exists(dir: &Path) -> Result<PathBuf, Error> {
	let dir = if dir.is_absolute() {
		dir.to_path_buf()
	} else {
		let mut cwd = env::current_dir().map_err(|_| Error::NoCwd)?;
		cwd.push(dir);

		cwd
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
