use crate::compiler::property::{Callback, Finalize, Function, Gc, Name, Output, Property, Trace};

use crate::compiler::{Compiler, Context, Runtime, Workspace};
use crate::input::Input;
use crate::Package;

use serde_json::Value;
use std::collections::HashMap;
use std::convert::TryFrom;

pub struct Include;

#[derive(Trace, Finalize)]
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

		let workspace: Workspace = (&package).into();

		let compiler = Compiler::new(&self.context, workspace);

		let compiler = {
			match input {
				None => compiler,
				Some(input) => compiler.prop(Box::new(Input(input))),
			}
		};

		let rendered = package
			.compile_with(compiler)
			.map_err(|err| err.to_string())?;

		Ok(rendered)
	}
}

impl Property for Include {
	fn generate(&self, runtime: Runtime) -> Output {
		let context = runtime.context;
		let params = vec![String::from("name"), String::from("input")];
		let handler = Handler { context };
		let function = Function {
			params,
			handler: Gc::new(Box::new(handler)),
		};

		let name = Name::Include;
		Output::Callback { name, function }
	}
}
