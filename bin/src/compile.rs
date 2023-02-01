use crate::error::Error;

use std::convert::TryFrom;
use std::env;
use std::path::{Path, PathBuf};

use anyhow::Result;
use clap::Parser;
use kct_compiler::Release;
use kct_helper::io;
use kct_helper::json::{merge, set_in};
use kct_kube::Filter;
use kct_package::Package;
use serde_json::{Map, Value};

#[derive(Parser)]
pub struct Args {
	#[arg(help = "directory with the KCP to compile")]
	package: PathBuf,
	#[arg(help = "set input values for the package", long, short)]
	input: Option<Vec<String>>,
	#[arg(help = "set specific values for the package", long, short)]
	set: Option<Vec<String>>,
	#[arg(help = "directory to save compiled yamls", long, short)]
	output: Option<PathBuf>,
	#[arg(help = "scope your package within a release", long)]
	release: Option<String>,
	#[arg(help = "comma separated paths to compile", long)]
	only: Option<String>,
	#[arg(help = "comma separated paths to not compile", long)]
	except: Option<String>,
}

pub fn run(args: Args) -> Result<String> {
	let inputs = {
		let mut from_files: Vec<Result<Value, Error>> = args
			.input
			.unwrap_or_default()
			.into_iter()
			.map(PathBuf::from)
			.map(|path| parse_input(&path))
			.collect();

		let from_sets: Vec<Result<Value, Error>> = args
			.set
			.unwrap_or_default()
			.into_iter()
			.map(|val| parse_set(&val))
			.collect();

		from_files.extend(from_sets);

		from_files
	};

	let input = merge_inputs(&inputs)?;

	let output = ensure_output_exists(&args.output)?;

	let package = Package::try_from(args.package.as_path())?;

	let release = args.release.map(|name| Release { name });

	let only: Vec<PathBuf> = args.only.map(|v| as_paths(&v)).unwrap_or_default();
	let except: Vec<PathBuf> = args.except.map(|v| as_paths(&v)).unwrap_or_default();
	let filter = Filter { only, except };

	let rendered = package.compile(input, release)?;

	let objects = kct_kube::find(&rendered, &filter)?;

	let documents: Vec<(PathBuf, String)> = objects
		.into_iter()
		.map(|(path, object)| (path, serde_yaml::to_string(&object).unwrap()))
		.collect();

	match output {
		None => {
			let stream: String = documents
				.into_iter()
				.map(|(_path, object)| object)
				.collect();

			Ok(stream)
		}
		Some(path) => {
			write_objects(&path, documents)?;

			Ok(format!("Objects written at \"{}\"", path.display()))
		}
	}
}

fn parse_input(path: &Path) -> Result<Value, Error> {
	let contents = if path == PathBuf::from("-") {
		io::from_stdin().map_err(|err| Error::InvalidInput("stdin".to_string(), err.to_string()))?
	} else {
		io::from_file(path)
			.map_err(|err| Error::InvalidInput(format!("{}", path.display()), err.to_string()))?
	};

	let file = path.to_str().unwrap();
	let parsed: Value = serde_json::from_str(&contents)
		.map_err(|err| Error::InvalidInput(file.to_string(), err.to_string()))?;

	Ok(parsed)
}

fn parse_set(val: &str) -> Result<Value, Error> {
	let (name, value) = {
		let parts = val.split('=').collect::<Vec<&str>>();

		(parts[0], parts[1])
	};

	let mut result = Value::Null;
	let path = name.split('.').collect::<Vec<&str>>();
	let value =
		serde_json::from_str(value).map_err(|err| Error::MalformedInput(err.to_string()))?;

	set_in(&mut result, &path, value);

	Ok(result)
}

fn merge_inputs(inputs: &[Result<Value, Error>]) -> Result<Option<Value>, Error> {
	if inputs.is_empty() {
		return Ok(None);
	}

	let mut base = Value::Object(Map::new());

	for value in inputs {
		match value {
			Err(err) => return Err(Error::MalformedInput(err.to_string())),
			Ok(input) => match input {
				Value::Object(_map) => merge(&mut base, input),
				_ => return Err(Error::MalformedInput("input is not object".to_string())),
			},
		}
	}

	Ok(Some(base))
}

fn ensure_output_exists(path: &Option<PathBuf>) -> Result<Option<PathBuf>, Error> {
	match path {
		None => Ok(None),
		Some(path) => {
			if path == &PathBuf::from("-") {
				return Ok(None);
			}

			let base = env::current_dir().map_err(|err| Error::InvalidOutput(err.to_string()))?;

			let path = io::ensure_dir_exists(&base, path)?;

			Ok(Some(path))
		}
	}
}

fn write_objects(root: &Path, objects: Vec<(PathBuf, String)>) -> Result<(), Error> {
	for (path, contents) in objects {
		let target = {
			let mut base = root.to_path_buf();

			base.push(path.strip_prefix("/").unwrap());
			base.with_extension("yaml")
		};

		io::write_contents(&target, &contents)?;
	}

	Ok(())
}

fn as_paths(paths: &str) -> Vec<PathBuf> {
	paths
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
		.collect()
}
