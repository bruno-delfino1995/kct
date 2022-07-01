use fs_extra::dir::{self as fsdir, CopyOptions};
use std::env;
use std::path::{Path, PathBuf};
pub use tempfile::TempDir;

pub fn tmp() -> TempDir {
	let temproot = match env::var("CARGO_TARGET_TMPDIR") {
		Ok(dir) => PathBuf::from(&dir),
		Err(_) => env::temp_dir(),
	};

	TempDir::new_in(&temproot)
		.unwrap_or_else(|_| panic!("Unable to create tempdir at: {}", &temproot.display()))
}

pub fn mk(at: &Path) {
	fsdir::create_all(at, true)
		.unwrap_or_else(|_| panic!("Unable to create directory at: {}", at.display()));
}

pub fn cp(source: &Path, target: &Path) {
	let mut options = CopyOptions::new();
	options.content_only = true;

	fsdir::copy(source, target, &options).unwrap_or_else(|_| {
		panic!(
			"Unable to copy dir from source({}) to target({})",
			source.display(),
			target.display()
		)
	});
}

pub fn mv(source: &Path, target: &Path) {
	let mut options = CopyOptions::new();
	options.content_only = true;

	fsdir::move_dir(source, target, &options).unwrap_or_else(|_| {
		panic!(
			"Unable to move dir from source({}) to target({})",
			source.display(),
			target.display()
		)
	});
}
