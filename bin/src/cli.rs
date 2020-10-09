mod compile;
mod package;

use clap::{App, ArgMatches};

pub fn create() -> App<'static, 'static> {
	App::new("Kubernetes Configuration Tool")
		.version("0.1.0")
		.about("K8S config without hideous templates or context babysitting")
		.subcommand(compile::command())
		.subcommand(package::command())
}

pub fn run<'a>(matches: (&str, Option<&ArgMatches<'a>>)) -> Result<String, String> {
	match matches {
		("compile", Some(matches)) => compile::run(matches),
		("package", Some(matches)) => package::run(matches),
		_ => Err(String::from("Unknown arguments combination")),
	}
}
