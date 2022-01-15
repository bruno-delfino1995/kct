use crate::compiler::Runtime;

use jrsonnet_evaluator::{
	error::Error as JrError, error::LocError, native::NativeCallback, FuncVal, Val,
};
use jrsonnet_parser::{Param, ParamsDesc};
use serde_json::Value;
use std::collections::HashMap;
use std::convert::From;
use std::hash::Hash;
use std::rc::Rc;

type Handler = Box<dyn Fn(HashMap<String, Value>) -> Result<Value, String> + 'static>;

#[derive(Clone)]
pub struct Function {
	pub params: Vec<String>,
	pub handler: Rc<Handler>,
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
				let name = name.as_str();
				let params = function.params.clone();
				let handler = function.handler.clone();

				let params_names = function.params.clone();
				let params_desc = {
					let names: Vec<Param> =
						params.into_iter().map(|n| Param(n.into(), None)).collect();

					ParamsDesc(Rc::new(names))
				};

				let handler = move |_caller, params: &[Val]| -> Result<Val, LocError> {
					let params_values = params.iter().map(|v| {
						Value::try_from(v)
							.expect("Extension functions should only receive valid JSON")
					});
					let params_names = params_names.clone().into_iter();

					let params = params_names.zip(params_values).collect();

					handler(params)
						.map(|v| Val::from(&v))
						.map_err(|err| LocError::new(JrError::RuntimeError(err.into())))
				};

				let callback = NativeCallback::new(params_desc, handler);

				let ext: Rc<FuncVal> = Rc::new(FuncVal::NativeExt(name.into(), Rc::new(callback)));

				Val::Func(ext)
			}
		}
	}
}

pub trait Property {
	fn generate(&self, runtime: Runtime) -> Output;
}
