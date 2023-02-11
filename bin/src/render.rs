use crate::error::Error;
use crate::operation::compile;

use std::path::PathBuf;

use anyhow::Result;
use clap::Parser;
use kct_cli::Output;
use kct_helper::io::{self, Location};
use kct_kube::Manifest;

#[derive(Parser)]
pub struct Args {
	#[arg(help = "directory to save compiled manifests", long, short)]
	output: Option<Output>,
	#[command(flatten)]
	compile: compile::Params,
}

pub fn run(args: Args) -> Result<String> {
	let kube = compile::run(args.compile)?;
	let manifests: Vec<Manifest> = kube.try_into()?;
	let documents: Vec<(PathBuf, String)> = manifests
		.into_iter()
		.map(|manifest| manifest.into())
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
