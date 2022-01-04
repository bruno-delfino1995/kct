use serde_json::{Map, Value};

use std::convert::From;

#[derive(Clone, Debug)]
pub struct Release {
	pub name: String,
}

impl From<Release> for Value {
	fn from(release: Release) -> Self {
		let mut map = Map::<String, Value>::new();
		map.insert(String::from("name"), Value::String(release.name));

		Value::Object(map)
	}
}
