use crate::cli::CliError;
use clap::ArgMatches;
use clap::{App, Arg, SubCommand};
use kct_helper::io;
use kct_helper::json::merge;
use kct_kube::Filter;
use kct_package::{Package, Release};
use serde_json::{Map, Value};
use std::env;
use std::error::Error;
use std::path::{Path, PathBuf};

pub fn command() -> App<'static, 'static> {
	SubCommand::with_name("compile")
		.about("compiles the package into valid k8s objects")
		.arg(
			Arg::with_name("package")
				.help("Target to run the subcommand onto")
				.index(1)
				.required(true),
		)
		.arg(
			Arg::with_name("input")
				.short("i")
				.long("input")
				.help("Sets the input values for the package")
				.multiple(true)
				.number_of_values(1)
				.takes_value(true),
		)
		.arg(
			Arg::with_name("output")
				.short("o")
				.long("output")
				.help("Where to save compiled yamls")
				.takes_value(true),
		)
		.arg(
			Arg::with_name("release")
				.long("release")
				.help("Scope your package within a release")
				.takes_value(true),
		)
		.arg(
			Arg::with_name("only")
				.long("only")
				.help("List of comma separated paths to compile")
				.takes_value(true),
		)
		.arg(
			Arg::with_name("except")
				.long("except")
				.help("List of comma separated path to not compile")
				.takes_value(true),
		)
}

pub fn run(matches: &ArgMatches) -> Result<String, Box<dyn Error>> {
	let inputs: Vec<Result<Value, String>> = matches
		.values_of("input")
		.unwrap_or_default()
		.map(PathBuf::from)
		.map(|path| parse_input(&path))
		.collect();
	let input = merge_inputs(&inputs).map_err(CliError::InvalidInput)?;

	let output: Option<PathBuf> = matches.value_of("output").map(PathBuf::from);
	let output = ensure_output_exists(&output)?;

	let package_from: PathBuf = matches.value_of("package").map(PathBuf::from).unwrap();
	let package = Package::from_path(package_from)?;

	let release = matches.value_of("release").map(|name| Release {
		name: String::from(name),
	});

	let only: Vec<PathBuf> = matches.value_of("only").map(as_paths).unwrap_or_default();
	let except: Vec<PathBuf> = matches.value_of("except").map(as_paths).unwrap_or_default();
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
		serde_json::from_str(&contents).map_err(|_err| format!("Unable to parse {}", file))?;

	Ok(parsed)
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
