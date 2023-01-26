use crate::{
	compiler::extension::{Extension, Name, Plugin},
	compiler::Runtime,
	Release,
};

use serde_json::{Map, Value};
use std::convert::From;

impl From<&Release> for Value {
	fn from(release: &Release) -> Self {
		let mut map = Map::<String, Value>::new();
		map.insert(String::from("name"), Value::String(release.name.clone()));

		Value::Object(map)
	}
}

impl Extension for Release {
	fn plug(&self, _: Runtime) -> Plugin {
		Plugin::Property {
			name: Name::Release,
			value: self.into(),
		}
	}
}
