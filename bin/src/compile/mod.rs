mod args;

use self::args::Output;

pub use self::args::Args;

use crate::error::Error;

use std::convert::TryFrom;
use std::path::PathBuf;

use anyhow::Result;
use kct_compiler::Release;
use kct_helper::io::{self, Location};
use kct_helper::json::merge;
use kct_kube::Filter;
use kct_package::Package;
use serde_json::{Map, Value};

pub fn run(args: Args) -> Result<String> {
	let input = {
		let mut inputs = args
			.input
			.unwrap_or_default()
			.into_iter()
			.map(|input| input.into())
			.collect::<Vec<Value>>();

		let sets = args
			.set
			.unwrap_or_default()
			.into_iter()
			.map(|set| set.into())
			.collect::<Vec<Value>>();

		inputs.extend(sets);

		merge_inputs(&inputs)?
	};

	let package = Package::try_from(args.package.as_path())?;

	let release = args.release.map(|name| Release { name });
	let rendered = package.compile(input, release)?;

	let only: Vec<PathBuf> = args.only.map(|v| v.into()).unwrap_or_default();
	let except: Vec<PathBuf> = args.except.map(|v| v.into()).unwrap_or_default();
	let filter = Filter { only, except };
	let objects = kct_kube::find(&rendered, &filter)?;

	let documents: Vec<(PathBuf, String)> = objects
		.into_iter()
		.map(|(path, object)| (path, serde_yaml::to_string(&object).unwrap()))
		.collect();

	let output = ensure_output_exists(&args.output)?;
	match output {
		out @ Location::Standard => {
			let contents = out.write(documents)?;

			Ok(contents)
		}
		out @ Location::Path(_) => {
			let path = out.write(documents)?;

			Ok(format!("Manifests written at \"{path}\""))
		}
	}
}

fn merge_inputs(inputs: &[Value]) -> Result<Option<Value>, Error> {
	if inputs.is_empty() {
		return Ok(None);
	}

	let mut base = Value::Object(Map::new());

	for input in inputs {
		match input {
			Value::Object(_map) => merge(&mut base, input),
			_ => return Err(Error::InvalidInput("input is not object".to_string())),
		}
	}

	Ok(Some(base))
}

fn ensure_output_exists(output: &Option<Output>) -> Result<Location, Error> {
	let location = output.as_ref().cloned().map(|out| out.into());

	match location {
		None | Some(Location::Standard) => Ok(Location::Standard),
		Some(Location::Path(path)) => {
			let path = io::ensure_dir_exists(&path)?;

			Ok(Location::Path(path))
		}
	}
}
