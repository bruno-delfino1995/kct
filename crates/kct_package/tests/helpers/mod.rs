mod fixture;

pub use self::fixture::Fixture;

use serde_json::Value;
use std::{env, path::PathBuf};
use tempfile::TempDir;
use tera::{Context, Tera};

pub fn json(contents: &str) -> Value {
	serde_json::from_str(contents).unwrap()
}

pub fn template(contents: &str, input: &Value) -> String {
	let context = Context::from_serialize(input).unwrap();

	Tera::one_off(contents, &context, true).unwrap()
}

pub fn tempdir() -> TempDir {
	let temproot = match env::var("CARGO_TARGET_TMPDIR") {
		Ok(dir) => PathBuf::from(&dir),
		Err(_) => env::temp_dir(),
	};

	TempDir::new_in(&temproot).expect(&format!(
		"Unable to create tempdir at: {}",
		&temproot.display()
	))
}
