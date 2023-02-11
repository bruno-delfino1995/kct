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

pub fn run(args: Args) -> Result<()> {
	let kube = compile::run(args.compile)?;
	let manifests: Vec<Manifest> = kube.try_into()?;
	let documents: Vec<(PathBuf, String)> = manifests
		.into_iter()
		.map(|manifest| manifest.into())
		.collect();

	let output = ensure_output_exists(&args.output)?;
	match output {
		out @ Location::Standard => out.write(documents)?,

		out @ Location::Path(_) => {
			out.write(documents)?;
			let path = args
				.output
				.and_then(|o| {
					let l: Location = o.into();

					l.path().map(|p| p.display().to_string())
				})
				.unwrap();

			println!("Manifests written at \"{path}\"");
		}
	}

	Ok(())
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
