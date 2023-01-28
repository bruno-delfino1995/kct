pub mod input;
pub mod release;

use crate::Runtime;

use std::collections::HashMap;
use std::fmt;
use std::hash::Hash;

use serde_json::Value;

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

pub trait Property {
	fn generate(&self, runtime: Runtime) -> Prop;
}

pub enum Prop {
	Primitive(Name, Value),
	Callable(Name, Function),
}

impl Prop {
	pub fn name(&self) -> &Name {
		match self {
			Prop::Primitive(n, _) => n,
			Prop::Callable(n, _) => n,
		}
	}
}

pub struct Function {
	pub params: Vec<String>,
	pub handler: Box<dyn Callback>,
}

impl fmt::Debug for Function {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "Function")
	}
}

pub trait Callback {
	fn call(&self, params: HashMap<String, Value>) -> Result<Value, String>;
}
