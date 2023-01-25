mod compile;
mod error;
mod package;

use clap::{Parser, Subcommand};
use std::fmt::Display;
use std::process;

#[derive(Parser)]
#[command(
	version,
	about = "K8s config without hideous templates or context babysitting",
	name = "Kubernetes Configuration Tool"
)]
#[command(
	disable_help_subcommand = true,
	help_expected = true,
	arg_required_else_help = true
)]
pub struct Cli {
	#[command(subcommand)]
	command: Command,
}

#[derive(Subcommand)]
pub enum Command {
	#[command(
		name = "compile",
		about = "Compiles the package into valid K8S objects"
	)]
	Compile(compile::Args),
	#[command(name = "package", about = "Packages the project for sharing")]
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
