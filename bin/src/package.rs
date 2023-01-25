use kct_package::Package;
use std::convert::TryFrom;
use std::error::Error;
use std::path::PathBuf;

use clap::Parser;

#[derive(Parser)]
pub struct Args {
	#[arg(help = "directory containing the KCP")]
	package: PathBuf,
}

pub fn run(args: Args) -> Result<String, Box<dyn Error>> {
	let package = Package::try_from(args.package.as_path())?;

	let cwd = std::env::current_dir()?;
	let compressed_path = package.archive(&cwd)?;

	Ok(format!(
		"Successfully packaged KCP and saved it to: {}",
		compressed_path.to_str().unwrap()
	))
}
