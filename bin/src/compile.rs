use crate::error::Error as CError;
use kct_helper::io;
use kct_helper::json::{merge, set_in};
use kct_kube::Filter;
use kct_package::{Package, Release};
use serde_json::{Map, Value};
use std::convert::TryFrom;
use std::env;
use std::error::Error;
use std::path::{Path, PathBuf};

use clap::Parser;

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

pub fn run(args: Args) -> Result<String, Box<dyn Error>> {
	let inputs = {
		let mut from_files: Vec<Result<Value, String>> = args
			.input
			.unwrap_or_default()
			.into_iter()
			.map(PathBuf::from)
			.map(|path| parse_input(&path))
			.collect();

		let from_sets: Vec<Result<Value, String>> = args
			.set
			.unwrap_or_default()
			.into_iter()
			.map(|val| parse_set(&val))
			.collect();

		from_files.extend(from_sets);

		from_files
	};

	let input = merge_inputs(&inputs).map_err(CError::InvalidInput)?;

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

fn parse_input(path: &Path) -> Result<Value, String> {
	let contents = if path == PathBuf::from("-") {
		io::from_stdin().map_err(|err| err.to_string())?
	} else {
		io::from_file(path).map_err(|err| err.to_string())?
	};

	let file = path.to_str().unwrap();
	let parsed: Value =
		serde_json::from_str(&contents).map_err(|_err| format!("Unable to parse {file}"))?;

	Ok(parsed)
}

fn parse_set(val: &str) -> Result<Value, String> {
	let (name, value) = {
		let parts = val.split('=').collect::<Vec<&str>>();

		(parts[0], parts[1])
	};

	let mut result = Value::Null;
	let path = name.split('.').collect::<Vec<&str>>();
	let value = serde_json::from_str(value)
		.map_err(|_err| format!("unable to parse value manually set for {name}"))?;

	set_in(&mut result, &path, value);

	Ok(result)
}

fn merge_inputs(inputs: &[Result<Value, String>]) -> Result<Option<Value>, String> {
	if inputs.is_empty() {
		return Ok(None);
	}

	let mut base = Value::Object(Map::new());

	for value in inputs {
		match value {
			Err(err) => return Err(err.to_owned()),
			Ok(input) => match input {
				Value::Object(_map) => merge(&mut base, input),
				_ => return Err(String::from("input must be an object")),
			},
		}
	}

	Ok(Some(base))
}

fn ensure_output_exists(path: &Option<PathBuf>) -> Result<Option<PathBuf>, String> {
	match path {
		None => Ok(None),
		Some(path) => {
			if path == &PathBuf::from("-") {
				return Ok(None);
			}

			let base = env::current_dir().map_err(|err| err.to_string())?;

			io::ensure_dir_exists(&base, path)
				.map_err(|err| err.to_string())
				.map(Some)
		}
	}
}

fn write_objects(root: &Path, objects: Vec<(PathBuf, String)>) -> Result<(), String> {
	for (path, contents) in objects {
		let target = {
			let mut base = root.to_path_buf();

			base.push(path.strip_prefix("/").unwrap());
			base.with_extension("yaml")
		};

		io::write_contents(&target, &contents).map_err(|err| err.to_string())?;
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
