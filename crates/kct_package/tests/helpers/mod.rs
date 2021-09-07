use serde_json::Value;
use tera::{Context, Tera};

pub fn json(contents: &str) -> Value {
	serde_json::from_str(contents).unwrap()
}

pub fn template(contents: &str, input: &Value) -> String {
	let context = Context::from_serialize(input).unwrap();

	Tera::one_off(&contents, &context, true).unwrap()
}
