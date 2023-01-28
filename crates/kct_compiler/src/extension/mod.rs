pub mod input;
pub mod release;

use crate::Runtime;

use std::collections::HashMap;
use std::fmt;
use std::hash::Hash;
use std::rc::Rc;

use serde_json::Value;

// TODO: Create extension for random struct that has name and value and then we create it and only
// after box it as an extension
// NOTE: Leverage the Property struct below

pub trait Extension {
	fn plug(&self, runtime: Runtime) -> Plugin;
}

pub struct Noop;

impl Extension for Noop {
	fn plug(&self, _: Runtime) -> Plugin {
		Plugin::Noop
	}
}

#[derive(Clone, Debug)]
pub enum Property {
	Primitive(Name, Value),
	Callable(Name, Function),
}

pub trait Predicate: Fn(&Value) -> Result<(), String> {}
impl<T: Fn(&Value) -> Result<(), String>> Predicate for T {}

impl Property {
	pub fn name(&self) -> &Name {
		match self {
			Property::Primitive(n, _) => n,
			Property::Callable(n, _) => n,
		}
	}
}

pub enum Plugin {
	Noop,
	Create(Property),
	Verify(Rc<dyn Predicate>),
}

impl Plugin {
	pub fn property(&self) -> Option<&Property> {
		match self {
			Plugin::Create(prop) => Some(prop),
			_ => None,
		}
	}

	pub fn validator(&self) -> Option<Rc<dyn Predicate>> {
		match self {
			Plugin::Verify(val) => Some(Rc::clone(val)),
			_ => None,
		}
	}
}

#[derive(Clone)]
pub struct Function {
	pub params: Vec<String>,
	pub handler: Rc<dyn Callback>,
}

impl fmt::Debug for Function {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "Function")
	}
}

pub trait Callback {
	fn call(&self, params: HashMap<String, Value>) -> Result<Value, String>;
}

#[derive(Clone, Copy, Hash, PartialEq, Eq, Debug)]
pub enum Name {
	File,
	Include,
	Input,
	Package,
	Release,
}

impl Name {
	// TODO: Create a defaults method with `Val::Null` for `Property` and `() -> error` for callbacks
	// How could we have these defaults registered by implementations instead of at definition level?
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

#[derive(Clone)]
pub struct Plugins {
	data: Vec<Rc<Plugin>>,
}

impl Default for Plugins {
	fn default() -> Self {
		Self::new()
	}
}

impl Plugins {
	pub fn new() -> Self {
		Self { data: Vec::new() }
	}

	pub fn put(&mut self, plugin: Plugin) {
		self.data.push(Rc::new(plugin));
	}

	pub fn iter(&self) -> impl Iterator<Item = &Rc<Plugin>> {
		self.data.iter()
	}
}
