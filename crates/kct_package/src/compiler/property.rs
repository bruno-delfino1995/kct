pub use jrsonnet_gc::{Finalize, Gc, Trace};

use crate::compiler::Runtime;

use jrsonnet_evaluator::{
	error::Error as JrError,
	error::LocError,
	native::{NativeCallback, NativeCallbackHandler},
	FuncVal, Val,
};
use jrsonnet_parser::{Param, ParamsDesc};
use serde_json::Value;
use std::collections::HashMap;
use std::convert::From;
use std::hash::Hash;
use std::rc::Rc;

#[derive(Clone, Trace, Finalize)]
pub struct Function {
	pub params: Vec<String>,
	pub handler: Gc<Box<dyn Callback>>,
}

pub trait Callback: Trace {
	fn call(&self, params: HashMap<String, Value>) -> Result<Value, String>;
}

impl NativeCallbackHandler for Function {
	fn call(
		&self,
		_from: Option<Rc<std::path::Path>>,
		args: &[Val],
	) -> jrsonnet_evaluator::error::Result<Val> {
		let names = self.params.clone().into_iter();
		let values = args.iter().map(|v| {
			Value::try_from(v).expect("Extension functions should only receive valid JSON")
		});

		let params = names.zip(values).collect();

		self.handler
			.call(params)
			.map(|v| Val::from(&v))
			.map_err(|err| LocError::new(JrError::RuntimeError(err.into())))
	}
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

#[derive(Clone)]
pub enum Output {
	Plain { name: Name, value: Value },
	Callback { name: Name, function: Function },
}

impl Output {
	pub fn name(&self) -> &Name {
		use Output::*;

		match self {
			Plain { name, .. } => name,
			Callback { name, .. } => name,
		}
	}
}

impl From<Output> for Val {
	fn from(original: Output) -> Self {
		use Output::*;

		match original {
			Plain { value, .. } => Val::from(&value),
			Callback { name, function } => {
				let params = function.params.clone();

				let params_desc = {
					let names: Vec<Param> =
						params.into_iter().map(|n| Param(n.into(), None)).collect();

					ParamsDesc(Rc::new(names))
				};

				let callback = NativeCallback::new(params_desc, Box::new(function));

				let name = name.as_str();
				let ext: Gc<FuncVal> = Gc::new(FuncVal::NativeExt(name.into(), Gc::new(callback)));

				Val::Func(ext)
			}
		}
	}
}

pub trait Property {
	fn generate(&self, runtime: Runtime) -> Output;
}
