use crate::extension::{Extension, Name, Plugin, Property};
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

impl Extension for Release {
	fn plug(&self, _: Runtime) -> Plugin {
		Plugin::Create(Property::Primitive(Name::Release, self.into()))
	}
}
