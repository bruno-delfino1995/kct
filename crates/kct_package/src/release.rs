use crate::compiler::{
	property::{Name, Output, Property},
	Runtime,
};

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
	fn generate(&self, _: Runtime) -> Output {
		Output::Plain {
			name: Name::Release,
			value: self.into(),
		}
	}
}
