mod compile;
mod package;

use clap::{App, ArgMatches};
use std::error::Error;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum CliError {
	#[error("Unknown command or args combination")]
	InvalidCall,
}

pub fn create() -> App<'static, 'static> {
	App::new("Kubernetes Configuration Tool")
		.version("0.1.0")
		.about("K8S config without hideous templates or context babysitting")
		.subcommand(compile::command())
		.subcommand(package::command())
}

pub fn run<'a>(matches: (&str, Option<&ArgMatches<'a>>)) -> Result<String, Box<dyn Error>> {
	match matches {
		("compile", Some(matches)) => compile::run(matches),
		("package", Some(matches)) => package::run(matches),
		_ => Err(CliError::InvalidCall.into()),
	}
}
