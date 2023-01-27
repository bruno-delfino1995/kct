use crate::Package;

use std::collections::HashMap;
use std::convert::TryFrom;
use std::rc::Rc;

use kct_compiler::extension::{Callback, Extension, Function, Name, Plugin};
use kct_compiler::{Compiler, Context, Input, Runtime, Target};
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

		let input: Option<Value> = params.get("input").cloned();

		let workspace: Target = (&package).into();

		let compiler = Compiler::new(&self.context, workspace);

		let compiler = {
			match input {
				None => compiler,
				Some(input) => compiler.extend(Box::new(Input(input))),
			}
		};

		let rendered = package
			.compile_with(compiler)
			.map_err(|err| err.to_string())?;

		Ok(rendered)
	}
}

impl Extension for Include {
	fn plug(&self, runtime: Runtime) -> Plugin {
		let context = runtime.context;
		let params = vec![String::from("name"), String::from("input")];
		let handler = Handler { context };
		let function = Function {
			params,
			handler: Rc::new(handler),
		};

		let name = Name::Include;
		Plugin::Callback { name, function }
	}
}
