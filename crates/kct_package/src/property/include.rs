use crate::Package;

use std::collections::HashMap;
use std::convert::TryFrom;

use kct_compiler::property::{Callback, Function, Name, Prop, Property};
use kct_compiler::{Compiler, Context, Input, Runtime};
use serde_json::Value;

pub struct Include;

struct Handler {
	context: Context,
}

impl Callback for Handler {
	fn call(&self, params: HashMap<String, Value>) -> Result<Value, String> {
		let name = match params.get("name") {
			None => return Err("name is required".into()),
			Some(name) => name,
		};

		let package = match name {
			Value::String(name) => name,
			_ => return Err("name should be a string".into()),
		};

		let root = self.context.vendor().join(package);
		let package = Package::try_from(root.as_path()).map_err(|err| err.to_string())?;

		let input = params.get("input").cloned().map(|v| (&Input(v)).into());

		let compiler = Compiler::inherit(&self.context)
			.with_static_prop(input)
			.with_target((&package).into());

		let rendered = package
			.compile_with(compiler)
			.map_err(|err| err.to_string())?;

		Ok(rendered)
	}
}

impl Property for Include {
	fn generate(&self, runtime: &Runtime) -> Prop {
		let context = runtime.context().clone();
		let params = vec![String::from("name"), String::from("input")];
		let handler = Handler { context };
		let function = Function {
			params,
			handler: Box::new(handler),
		};

		let name = Name::Include;
		Prop::Callable(name, function)
	}

	fn name(&self) -> Name {
		Name::Include
	}
}
