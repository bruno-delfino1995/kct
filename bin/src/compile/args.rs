use crate::error::Error;

use std::convert::Infallible;

use std::path::PathBuf;
use std::str::FromStr;

use clap::Parser;

use kct_helper::io::{self, Location};
use kct_helper::json::set_in;

use serde_json::Value;

#[derive(Parser)]
pub struct Args {
	#[arg(help = "directory with the package to compile")]
	pub package: PathBuf,
	#[arg(help = "set multiple values for the package", long, short)]
	pub input: Option<Vec<Input>>,
	#[arg(help = "set specific parameters for the package", long, short)]
	pub set: Option<Vec<Set>>,
	#[arg(help = "directory to save compiled manifests", long, short)]
	pub output: Option<Output>,
	#[arg(help = "scope your package within a release", long)]
	pub release: Option<String>,
	#[arg(help = "comma separated paths to compile", long)]
	pub only: Option<Paths>,
	#[arg(help = "comma separated paths to not compile", long)]
	pub except: Option<Paths>,
}

#[derive(Clone)]
pub struct Input(Value);

impl FromStr for Input {
	type Err = Error;

	fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
		let location = Location::from_str(s).map_err(|err| Error::InvalidInput(err.to_string()))?;
		let contents = location
			.read()
			.map_err(|err| Error::InvalidInput(err.to_string()))?;
		let parsed: Value =
			serde_json::from_str(&contents).map_err(|err| Error::InvalidInput(err.to_string()))?;

		Ok(Self(parsed))
	}
}

impl From<Input> for Value {
	fn from(val: Input) -> Self {
		val.0
	}
}

#[derive(Clone)]
pub struct Set(Value);

impl FromStr for Set {
	type Err = Error;

	fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
		let (name, value) = {
			let parts = s.split('=').collect::<Vec<&str>>();

			(parts[0], parts[1])
		};

		let mut result = Value::Null;
		let path = name.split('.').collect::<Vec<&str>>();
		let value =
			serde_json::from_str(value).map_err(|err| Error::InvalidInput(err.to_string()))?;

		set_in(&mut result, &path, value);

		Ok(Self(result))
	}
}

impl From<Set> for Value {
	fn from(val: Set) -> Self {
		val.0
	}
}

#[derive(Clone)]
pub struct Output(Location);

impl FromStr for Output {
	type Err = Error;

	fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
		let location = Location::from_str(s).or_else(|err| match err {
			io::Error::NotFound(path) => Ok(Location::Path(path)),
			err => Err(err),
		})?;

		Ok(Self(location))
	}
}

impl From<Output> for Location {
	fn from(val: Output) -> Self {
		val.0
	}
}

#[derive(Clone)]
pub struct Paths(Vec<PathBuf>);

impl FromStr for Paths {
	type Err = Infallible;

	fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
		let paths = s
			.trim()
			.split(',')
			.map(|path| path.trim())
			.filter(|str| !str.is_empty())
			.map(|path| path.split('.'))
			.map(|path| {
				let mut base = PathBuf::from("/");
				for part in path {
					base.push(part);
				}

				base
			})
			.collect();

		Ok(Self(paths))
	}
}

impl From<Paths> for Vec<PathBuf> {
	fn from(val: Paths) -> Self {
		val.0
	}
}
