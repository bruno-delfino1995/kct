pub mod input;
pub mod release;

use crate::Runtime;

use serde_json::Value;
use std::collections::HashMap;
use std::hash::Hash;
use std::rc::Rc;

pub trait Extension {
	fn plug(&self, runtime: Runtime) -> Plugin;
}

#[derive(Clone)]
pub enum Plugin {
	Property { name: Name, value: Value },
	Callback { name: Name, function: Function },
}

impl Plugin {
	pub fn name(&self) -> &Name {
		use Plugin::*;

		match self {
			Property { name, .. } => name,
			Callback { name, .. } => name,
		}
	}

	pub fn is_property(&self) -> bool {
		use Plugin::*;

		match self {
			Property { .. } => true,
			Callback { .. } => false,
		}
	}

	pub fn is_callback(&self) -> bool {
		!self.is_property()
	}
}

#[derive(Clone)]
pub struct Function {
	pub params: Vec<String>,
	pub handler: Rc<dyn Callback>,
}

pub trait Callback {
	fn call(&self, params: HashMap<String, Value>) -> Result<Value, String>;
}

#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub enum Name {
	File,
	Include,
	Input,
	Package,
	Release,
}

impl Name {
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
	data: HashMap<Name, Rc<Plugin>>,
}

impl Default for Plugins {
	fn default() -> Self {
		Self::new()
	}
}

impl Plugins {
	pub fn new() -> Self {
		Self {
			data: HashMap::new(),
		}
	}

	pub fn put(&mut self, plugin: Plugin) {
		let name = plugin.name();

		self.data.insert(*name, Rc::new(plugin));
	}

	pub fn get(&self, name: Name) -> Option<Rc<Plugin>> {
		self.data.get(&name).map(Rc::clone)
	}

	pub fn properties(&self) -> Plugins {
		let data = self
			.data
			.iter()
			.filter(|(_, p)| p.is_property())
			.map(|(n, p)| (*n, p.clone()))
			.collect();

		Self { data }
	}

	pub fn callbacks(&self) -> Plugins {
		let data = self
			.data
			.iter()
			.filter(|(_, p)| p.is_callback())
			.map(|(n, p)| (*n, p.clone()))
			.collect();

		Self { data }
	}
}
