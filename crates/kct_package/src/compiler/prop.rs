pub use jrsonnet_gc::{unsafe_empty_trace, Finalize, Gc, Trace};

use crate::compiler::extension::{Function, Plugin};

use jrsonnet_evaluator::{
	error::Error as JrError,
	error::LocError,
	native::{NativeCallback, NativeCallbackHandler},
	FuncVal, Val,
};
use jrsonnet_parser::{Param, ParamsDesc};
use serde_json::Value;
use std::convert::From;
use std::rc::Rc;

impl Finalize for Function {}
unsafe impl Trace for Function {
	unsafe_empty_trace!();
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

impl From<Plugin> for Val {
	fn from(original: Plugin) -> Self {
		use Plugin::*;

		match original {
			Property { value, .. } => Val::from(&value),
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
