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
