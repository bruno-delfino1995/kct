mod compile;
mod error;
mod package;

use clap::{AppSettings, Parser, Subcommand};
use std::fmt::Display;
use std::process;

#[derive(Parser)]
#[clap(
	version,
	about = "K8s config without hideous templates or context babysitting",
	name = "Kubernetes Configuration Tool"
)]
#[clap(global_setting(AppSettings::DisableHelpSubcommand))]
#[clap(global_setting(AppSettings::ArgRequiredElseHelp))]
#[clap(global_setting(AppSettings::DeriveDisplayOrder))]
pub struct Cli {
	#[clap(subcommand)]
	command: Command,
}

#[derive(Subcommand)]
pub enum Command {
	#[clap(
		name = "compile",
		about = "Compiles the package into valid k8s objects"
	)]
	Compile(compile::Args),
	#[clap(name = "package", about = "Package a KCP into a KCP Archive")]
	Package(package::Args),
}

fn main() {
	let cli = Cli::parse();

	let result = match cli.command {
		Command::Compile(args) => compile::run(args),
		Command::Package(args) => package::run(args),
	};

	let output = result.unwrap_or_else(exit);

	println!("{}", output)
}

fn exit<T: Display, R>(err: T) -> R {
	eprintln!("{}", err);
	process::exit(1)
}
