mod file;
mod include;

pub use self::file::File;
pub use self::include::Include;

use crate::compiler::Compiler;

use jrsonnet_evaluator::native::NativeCallback;
use std::hash::Hash;

#[derive(Clone, Hash, PartialEq, Eq)]
pub enum Name {
	File,
	Include,
}

impl Name {
	pub fn as_str(&self) -> &str {
		use Name::*;

		match self {
			File => "files",
			Include => "include",
		}
	}
}

pub trait Extension {
	fn name(&self) -> Name;

	fn generate(&self, compiler: &Compiler) -> NativeCallback;
}
