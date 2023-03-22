use crate::Package;

use std::collections::HashMap;
use std::convert::TryFrom;

use kct_compiler::property::{Callback, Generator, Name, Prop};
use kct_compiler::{Compiler, Context, Input, Runtime, Trace};
use serde_json::Value;

pub struct Include;

#[derive(Trace)]
struct Handler {
	context: Context,
}

impl Callback for Handler {
	fn call(&self, params: HashMap<String, Value>) -> Result<Value, String> {
		let name = params.get("name").unwrap();
		let package = match name {
			Value::String(name) => name,
			_ => return Err("name should be a string".into()),
		};

		let root = self.context.vendor().join(package);
		let package = Package::try_from(root.as_path()).map_err(|err| err.to_string())?;

		let prop = params.get("input").cloned().map(|v| (&Input(v)).into());
		let compiler = Compiler::new(&self.context)
			.with_static_prop(prop)
			.with_target((&package).into());

		let rendered = package
			.compile_with(compiler)
			.map_err(|err| err.to_string())?;

		Ok(rendered)
	}
}

impl Generator for Include {
	fn generate(&self, runtime: &Runtime) -> Prop {
		let context = runtime.context().clone();
		let params = vec![String::from("name"), String::from("input")];
		let handler = Handler { context };

		Prop::callable(Name::Include, params, handler)
	}

	fn name(&self) -> Name {
		Name::Include
	}
}
