use crate::serde::W;

use std::collections::HashMap;
use std::convert::From;
use std::fmt;

use jrsonnet_evaluator::error::Error;
use jrsonnet_evaluator::error::ErrorKind;
use jrsonnet_evaluator::function::builtin::{Builtin, BuiltinParam};
use jrsonnet_evaluator::function::parse::parse_builtin_call;
use jrsonnet_evaluator::function::{ArgsLike, CallLocation, FuncVal};
use jrsonnet_evaluator::gc::TraceBox;
use jrsonnet_evaluator::{Context, Val};
use jrsonnet_gcmodule::{Cc, Trace};
use serde_json::Value;

pub enum Property {
	Primitive(Value),
	Callable(Function),
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
			Property::Callable(func) => write!(f, "{}({})", func.name, func.params().join(", ")),
		}
	}
}

#[derive(Trace)]
pub struct Function {
	name: String,
	params: Vec<BuiltinParam>,
	handler: TraceBox<dyn Callback>,
}

impl Function {
	pub fn new(name: String, params: Vec<String>, handler: impl Callback) -> Self {
		let params = params
			.into_iter()
			.map(|p| -> BuiltinParam {
				BuiltinParam {
					name: Some(p.into()),
					has_default: false,
				}
			})
			.collect();

		Self {
			name,
			params,
			handler: TraceBox(Box::new(handler)),
		}
	}

	pub fn params(&self) -> Vec<String> {
		self.params
			.iter()
			.map(|p| p.name.as_ref().unwrap().to_string())
			.collect()
	}
}

pub trait Callback: Send + Trace {
	fn call(&self, params: HashMap<String, Value>) -> std::result::Result<Value, String>;
}

impl Builtin for Function {
	fn name(&self) -> &str {
		&self.name
	}

	fn params(&self) -> &[BuiltinParam] {
		&self.params
	}

	fn call(
		&self,
		ctx: Context,
		_: CallLocation<'_>,
		args: &dyn ArgsLike,
	) -> jrsonnet_evaluator::Result<Val> {
		let args = parse_builtin_call(ctx, &self.params, args, true)?;
		let args = args
			.into_iter()
			.map(|a| a.expect("natives have no default params"))
			.map(|a| a.evaluate())
			.collect::<jrsonnet_evaluator::Result<Vec<Val>>>()?;

		let names = self.params().into_iter();
		let values = args.iter().map(|val| {
			W(val)
				.try_into()
				.expect("Extension functions should only receive valid JSON")
		});

		let params = names.zip(values).collect();

		self.handler
			.call(params)
			.map(|value| {
				let wrapped: W<Val> = (&value).into();

				wrapped.0
			})
			.map_err(|err| Error::new(ErrorKind::RuntimeError(err.into())))
	}
}

impl From<Property> for Val {
	fn from(original: Property) -> Self {
		match original {
			Property::Primitive(value) => {
				let wrapped: W<Val> = (&value).into();

				wrapped.0
			}
			Property::Callable(function) => {
				let ext = FuncVal::Builtin(Cc::new(TraceBox(Box::new(function))));

				Val::Func(ext)
			}
		}
	}
}
