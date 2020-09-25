#![feature(move_ref_pattern)]

mod cli;

use std::fmt::Display;
use std::process;

fn main() {
	let matches = cli::create().get_matches();

	let result = match matches.subcommand() {
		("compile", Some(matches)) => cli::compile::run(matches),
		_ => exit("Unknown arguments combination"),
	};

	let result: String = result.unwrap_or_else(exit);

	println!("{}", result);
}

fn exit<T: Display, R>(err: T) -> R {
	eprintln!("{}", err);
	process::exit(1)
}
