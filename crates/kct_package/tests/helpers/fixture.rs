use crate::helpers;

use fs_extra::dir::{self as fsdir, CopyOptions};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

pub struct Fixture {}

impl Fixture {
	fn original() -> PathBuf {
		let root_dir =
			env::var("CARGO_MANIFEST_DIR").expect("Unable to read env var: $CARGO_MANIFEST_DIR");

		let mut source = PathBuf::from(&root_dir);
		source.push("tests/fixtures/kcp");
		source
	}

	pub fn custom(with: Vec<(&str, &str)>, without: Vec<&str>) -> TempDir {
		let dir = {
			let tempdir = helpers::tempdir();
			let source = Self::original();

			let mut options = CopyOptions::new();
			options.content_only = true;
			fsdir::copy(&source, &tempdir.path(), &options).expect(&format!(
				"Unable to copy dir from source({}) to target({})",
				&source.display(),
				&tempdir.path().display()
			));

			tempdir
		};

		for (path, contents) in with {
			let to_add = dir.path().join(path);
			let parent = to_add.parent().expect("It should have tempdir as parent");

			if !parent.exists() {
				fsdir::create_all(parent, false).expect(&format!(
					"Unable to create parent at: {}",
					&parent.display()
				));
			}

			fs::write(&to_add, contents)
				.expect(&format!("Unable to write file at: {}", &to_add.display()));
		}

		for path in without {
			let to_remove = dir.path().join(path);

			if to_remove.is_dir() {
				fs::remove_dir_all(&to_remove).expect(&format!(
					"Unable to remove dir at: {}",
					&to_remove.display()
				));
			} else {
				fs::remove_file(&to_remove).expect(&format!(
					"Unable to delete file at: {}",
					&to_remove.display()
				));
			}
		}

		dir
	}

	pub fn contents<T: AsRef<Path>>(path: T) -> String {
		let mut at = Self::original();
		at.push(path);

		fs::read_to_string(&at).expect(&format!("Unable to read contents from: {}", &at.display()))
	}
}
