use crate::compiler::Compiler;

use serde_json::Value;
use std::{collections::HashMap, hash::Hash};

type Handler = Box<dyn Fn(HashMap<String, Value>) -> Result<Value, String> + 'static>;

pub struct Function {
	pub params: Vec<String>,
	pub handler: Handler,
}

#[derive(Clone, Hash, PartialEq, Eq)]
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

pub enum Output {
	Plain(Value),
	Callback(Function),
}

pub trait Property {
	fn name(&self) -> Name;

	fn generate(&self, compiler: &Compiler) -> Output;
}
