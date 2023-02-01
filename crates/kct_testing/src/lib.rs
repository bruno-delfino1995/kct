mod fixture;

pub mod dir;
pub mod io;

pub use self::fixture::Fixture;

use kct_package::Package;
use serde_json::Value;
use tera::{Context, Tera};

pub use serde_json::json;

pub fn json(contents: &str) -> Value {
	serde_json::from_str(contents).unwrap()
}

pub fn template(contents: &str, input: &Value) -> String {
	let context = Context::from_serialize(input).unwrap();

	Tera::one_off(contents, &context, true).unwrap()
}

pub fn compile(main: &str) -> Value {
	let custom = vec![("templates/main.jsonnet", main)];
	let dir = Fixture::custom(custom, vec![]);
	let pkg = Package::try_from(dir.path()).unwrap();

	let input = pkg.example.clone().unwrap();
	let output = pkg.compile(Some(input), None).unwrap();

	drop(dir);
	output
}
