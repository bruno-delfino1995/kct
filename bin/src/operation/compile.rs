use crate::error::Error;

use std::convert::TryFrom;
use std::path::PathBuf;

use anyhow::Result;
use clap::Parser;
use kct_cli::{Input, Paths, Set};
use kct_compiler::Release;
use kct_helper::json::merge;
use kct_kube::Kube;
use kct_package::Package;
use serde_json::{Map, Value};

#[derive(Parser, Clone)]
pub struct Params {
	#[arg(help = "directory with the package to compile")]
	package: PathBuf,
	#[arg(help = "set multiple values for the package", long, short)]
	input: Option<Vec<Input>>,
	#[arg(help = "set specific parameters for the package", long, short)]
	set: Option<Vec<Set>>,
	#[arg(help = "scope your package within a release", long)]
	release: Option<String>,
	#[arg(help = "comma separated paths to compile", long)]
	only: Option<Paths>,
	#[arg(help = "comma separated paths to not compile", long)]
	except: Option<Paths>,
}

pub fn run(args: Params) -> Result<Kube> {
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

	let kube = Kube::builder()
		.only(only)
		.except(except)
		.value(rendered)
		.build()?;

	Ok(kube)
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
