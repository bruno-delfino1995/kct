use std::collections::HashMap;
use std::convert::From;
use std::fmt;
use std::path::Path;
use std::rc::Rc;

use jrsonnet_evaluator::error::{Error as JrError, LocError};
use jrsonnet_evaluator::native::{NativeCallback, NativeCallbackHandler};
use jrsonnet_evaluator::{FuncVal, Val};
use jrsonnet_gc::{unsafe_empty_trace, Finalize, Gc, Trace};
use jrsonnet_parser::{Param, ParamsDesc};
use serde_json::Value;

pub enum Property {
	Primitive(Value),
	Callable(String, Function),
}

impl Property {
	pub fn value(&self) -> Option<&Value> {
		match self {
			Property::Primitive(v) => Some(v),
			_ => None,
		}
	}
}

impl fmt::Debug for Property {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Property::Primitive(val) => write!(f, "{val:?}"),
			Property::Callable(name, func) => write!(f, "{}({})", name, func.params.join(", ")),
		}
	}
}

pub struct Function {
	pub params: Vec<String>,
	pub handler: Box<dyn Callback>,
}

pub trait Callback: Send {
	fn call(&self, params: HashMap<String, Value>) -> Result<Value, String>;
}

impl Finalize for Function {}
unsafe impl Trace for Function {
	unsafe_empty_trace!();
}

impl NativeCallbackHandler for Function {
	fn call(
		&self,
		_from: Option<Rc<Path>>,
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

impl From<Property> for Val {
	fn from(original: Property) -> Self {
		match original {
			Property::Primitive(value) => Val::from(&value),
			Property::Callable(name, function) => {
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
