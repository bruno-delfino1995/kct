use crate::Package;

use crate::compiler::extension::{Extension, Name, Plugin};
use crate::compiler::Runtime;

use serde_json::{Map, Value};

impl From<&Package> for Value {
	fn from(package: &Package) -> Self {
		let mut map = Map::<String, Value>::new();
		map.insert(
			String::from("name"),
			Value::String(package.spec.name.clone()),
		);
		map.insert(
			String::from("version"),
			Value::String(package.spec.version.to_string()),
		);

		Value::Object(map)
	}
}

impl Extension for Package {
	fn plug(&self, _: Runtime) -> Plugin {
		Plugin::Property {
			name: Name::Package,
			value: self.into(),
		}
	}
}
