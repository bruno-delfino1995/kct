mod file;
mod include;

pub use file::File;
pub use include::Include;

use crate::Compiler;
use derive_builder::Builder;
use jrsonnet_evaluator::native::NativeCallback;
use jrsonnet_evaluator::FuncVal;
use jrsonnet_evaluator::{
	error::Error as JrError,
	error::LocError,
	trace::{ExplainingFormat, PathResolver},
	EvaluationState, ManifestFormat, Val,
};
use serde_json::Value;
use std::collections::HashMap;
use std::convert::From;
use std::hash::Hash;
use std::path::{Path, PathBuf};
use std::rc::Rc;

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
