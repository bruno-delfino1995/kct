use crate::property::{Name, Prop};
use crate::Release;

use std::convert::From;

use serde_json::{Map, Value};

impl From<&Release> for Value {
	fn from(release: &Release) -> Self {
		let mut map = Map::<String, Value>::new();
		map.insert(String::from("name"), Value::String(release.name.clone()));

		Value::Object(map)
	}
}

impl From<&Release> for Prop {
	fn from(val: &Release) -> Self {
		Prop::primitive(Name::Release, val.into())
	}
}
