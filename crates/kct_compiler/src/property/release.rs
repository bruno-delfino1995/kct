use crate::property::{Name, Prop, Property};
use crate::{Release, Runtime};

use std::convert::From;

use serde_json::{Map, Value};

impl From<&Release> for Value {
	fn from(release: &Release) -> Self {
		let mut map = Map::<String, Value>::new();
		map.insert(String::from("name"), Value::String(release.name.clone()));

		Value::Object(map)
	}
}

impl Property for Release {
	fn generate(&self, _: Runtime) -> Prop {
		Prop::Primitive(Name::Release, self.into())
	}
}
