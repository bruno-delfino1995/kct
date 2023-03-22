pub mod input;
pub mod release;

use crate::Runtime;

use std::hash::Hash;

use serde_json::Value;

pub use kct_jsonnet::property::{Callback, Function, Property};

#[derive(Clone, Copy, Hash, PartialEq, Eq, Debug)]
pub enum Name {
	Files,
	Include,
	Input,
	Package,
	Release,
}

impl Name {
	pub fn all() -> [Name; 5] {
		use Name::*;

		[Files, Include, Input, Package, Release]
	}

	pub fn as_str(&self) -> &str {
		use Name::*;

		match self {
			Files => "files",
			Include => "include",
			Package => "package",
			Release => "release",
			Input => "input",
		}
	}
}

pub trait Generator {
	fn name(&self) -> Name;

	fn generate(&self, runtime: &Runtime) -> Prop;
}

pub struct Prop(Name, Property);

impl Prop {
	pub fn primitive(name: Name, value: Value) -> Self {
		Prop(name, Property::Primitive(value))
	}

	pub fn callable(name: Name, params: Vec<String>, handler: impl Callback) -> Self {
		let func = Function::new(name.as_str().to_string(), params, handler);

		Prop(name, Property::Callable(func))
	}

	pub fn take(self) -> (Name, Property) {
		(self.0, self.1)
	}

	pub fn name(&self) -> &Name {
		&self.0
	}

	pub fn value(&self) -> Option<&Value> {
		self.1.value()
	}
}
