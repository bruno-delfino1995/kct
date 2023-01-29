pub mod input;
pub mod release;

use crate::Runtime;

use std::hash::Hash;

use serde_json::Value;

pub use kct_jsonnet::property::{Callback, Function, Property};

#[derive(Clone, Copy, Hash, PartialEq, Eq, Debug)]
pub enum Name {
	File,
	Include,
	Input,
	Package,
	Release,
}

impl Name {
	// TODO: Create a defaults method with `Val::Null` for `Primitive` and `() -> error` for
	// `Callable`. How could we have these defaults registered by implementations instead of at
	// definition level?
	pub fn all() -> [Name; 5] {
		use Name::*;

		[File, Include, Input, Package, Release]
	}

	pub fn as_str(&self) -> &str {
		use Name::*;

		match self {
			File => "files",
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

	pub fn callable(name: Name, func: Function) -> Self {
		Prop(name, Property::Callable(name.as_str().to_string(), func))
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
