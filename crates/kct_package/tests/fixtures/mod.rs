use std::env;
use std::fs;
use std::path::{Path, PathBuf};

pub struct Fixture {}

impl Fixture {
	fn root() -> PathBuf {
		let root_dir = &env::var("CARGO_MANIFEST_DIR").expect("$CARGO_MANIFEST_DIR");

		let mut source = PathBuf::from(root_dir);
		source.push("tests/fixtures");
		source
	}

	pub fn path<T: AsRef<Path>>(name: T) -> PathBuf {
		let mut source = Self::root();
		source.push(name.as_ref());

		source
	}

	pub fn contents<T: AsRef<Path>>(path: T) -> String {
		let at = Self::path(path);

		fs::read_to_string(at).unwrap()
	}
}
