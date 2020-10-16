use serde_json::Value;
use tera::{Context, Tera};

pub fn values(contents: &str) -> Value {
	serde_json::from_str(contents).unwrap()
}

pub fn template(contents: &str, values: &Value) -> String {
	let context = Context::from_serialize(values).unwrap();

	Tera::one_off(&contents, &context, true).unwrap()
}
