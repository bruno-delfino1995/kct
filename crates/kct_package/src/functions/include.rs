use crate::compiler::property::{Callback, Finalize, Function, Gc, Name, Output, Property, Trace};
use crate::compiler::workspace::WorkspaceBuilder;
use crate::compiler::{Compiler, Runtime};
use crate::input::Input;
use crate::Package;

use serde_json::Value;
use std::collections::HashMap;
use std::convert::TryFrom;
use std::path::PathBuf;

pub struct Include;

#[derive(Trace, Finalize)]
struct Handler {
	vendor: PathBuf,
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

		let root = self.vendor.join(package);
		let package = Package::try_from(root.as_path()).map_err(|err| err.to_string())?;

		let input: Option<Value> = params.get("input").cloned();

		let workspace_builder: WorkspaceBuilder = (&package).into();

		let compiler: Compiler = workspace_builder
			.vendor(self.vendor.clone())
			.build()?
			.into();

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
		let vendor = runtime.workspace.vendor().to_path_buf();

		let params = vec![String::from("name"), String::from("input")];
		let handler = Handler { vendor };
		let function = Function {
			params,
			handler: Gc::new(Box::new(handler)),
		};

		let name = Name::Include;
		Output::Callback { name, function }
	}
}
