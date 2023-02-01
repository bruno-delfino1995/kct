use crate::dir::{self, TempDir};
use crate::io;

use std::env;
use std::fs;
use std::path::{Path, PathBuf};

pub struct Fixture {}

impl Fixture {
	fn original() -> PathBuf {
		let root_dir = env!("CARGO_MANIFEST_DIR");

		let mut source = PathBuf::from(root_dir);
		source.push("tests/kcp");
		source
	}

	pub fn custom(with: Vec<(&str, &str)>, without: Vec<&str>) -> TempDir {
		let dir = {
			let tempdir = dir::tmp();
			let source = Self::original();

			dir::cp(&source, tempdir.path());

			tempdir
		};

		for (path, contents) in with {
			let to_add = dir.path().join(path);
			let parent = to_add.parent().expect("It should have tempdir as parent");

			if !parent.exists() {
				dir::mk(parent);
			}

			fs::write(&to_add, contents)
				.unwrap_or_else(|_| panic!("Unable to write file at: {}", &to_add.display()));
		}

		for path in without {
			let to_remove = dir.path().join(path);

			io::rm(&to_remove);
		}

		dir
	}

	pub fn contents<T: AsRef<Path>>(path: T) -> String {
		let mut at = Self::original();
		at.push(path);

		fs::read_to_string(&at)
			.unwrap_or_else(|_| panic!("Unable to read contents from: {}", &at.display()))
	}
}
