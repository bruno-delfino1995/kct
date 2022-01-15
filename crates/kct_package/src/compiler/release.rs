use crate::property::{Name, Property};
use serde_json::{Map, Value};

use std::convert::From;

#[derive(Clone, Debug)]
pub struct Release {
	pub name: String,
}

impl From<&Release> for Value {
	fn from(release: &Release) -> Self {
		let mut map = Map::<String, Value>::new();
		map.insert(String::from("name"), Value::String(release.name.clone()));

		Value::Object(map)
	}
}

impl Property for Release {
	fn name(&self) -> Name {
		Name::Release
	}

	fn generate(&self) -> Value {
		self.into()
	}
}
