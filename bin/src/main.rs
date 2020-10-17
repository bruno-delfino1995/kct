mod cli;

use std::fmt::Display;
use std::process;

fn main() {
	let matches = cli::create().get_matches();

	let result = cli::run(matches.subcommand()).unwrap_or_else(exit);

	println!("{}", result);
}

fn exit<T: Display, R>(err: T) -> R {
	eprintln!("{}", err);
	process::exit(1)
}
