use serde_json::Value;
use std::hash::Hash;

#[derive(Clone, Hash, PartialEq, Eq)]
pub enum Name {
	Package,
	Release,
	Input,
}

impl Name {
	pub fn as_str(&self) -> &str {
		use Name::*;

		match self {
			Package => "package",
			Release => "release",
			Input => "input",
		}
	}
}

pub trait Property {
	fn name(&self) -> Name;

	fn generate(&self) -> Value;
}
