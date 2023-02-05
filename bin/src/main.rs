mod compile;
mod error;

use std::fmt;
use std::process;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
	version,
	about = "Kubernetes templates without hideous language or context babysitting",
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
	#[command(name = "compile", about = "Compiles package into valid manifests")]
	Compile(compile::Args),
}

fn main() {
	let cli = Cli::parse();

	let result = match cli.command {
		Command::Compile(args) => compile::run(args),
	};

	let output = result.unwrap_or_else(exit);

	println!("{output}")
}

fn exit<T: fmt::Debug, R>(err: T) -> R {
	eprintln!("{err:?}");
	process::exit(1)
}
