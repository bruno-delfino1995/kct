use std::env;
use std::fs;
use std::path::PathBuf;

pub struct Fixture {}

impl Fixture {
	pub fn dir(with: &[&str]) -> PathBuf {
		let root_dir = &env::var("CARGO_MANIFEST_DIR").expect("$CARGO_MANIFEST_DIR");

		let sources = with.iter().map(|name| {
			let mut source = PathBuf::from(root_dir);
			source.push("tests/fixtures");
			source.push(name);
			source
		});

		let tempdir = tempfile::tempdir().unwrap();
		let dir = tempdir.path();
		let targets = with.iter().map(|name| {
			let mut target = PathBuf::from(dir);
			target.push(name);
			target
		});

		for (source, target) in sources.zip(targets) {
			fs::copy(source, target).unwrap();
		}

		tempdir.into_path()
	}
}
