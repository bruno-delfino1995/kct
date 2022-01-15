use crate::compiler::{
	property::{Function, Name, Output, Property},
	Compiler,
};
use crate::input::Input;
use crate::Package;

use serde_json::Value;
use std::collections::HashMap;
use std::convert::TryFrom;

pub struct Include;

impl Property for Include {
	fn name(&self) -> Name {
		Name::Include
	}

	fn generate(&self, compiler: &Compiler) -> Output {
		let params = vec![String::from("name"), String::from("input")];

		let vendor = compiler.workspace.vendor.to_path_buf();
		let workspace = compiler.workspace.clone();
		let compiler = compiler.clone();

		let handler = Box::new(
			move |params: HashMap<String, Value>| -> Result<Value, String> {
				let name = match params.get("name") {
					None => return Err("name is required".into()),
					Some(name) => name,
				};

				let package = match name {
					Value::String(name) => name,
					_ => return Err("name should be a string".into()),
				};

				let root = vendor.join(&package);
				let package = Package::try_from(root).map_err(|err| err.to_string())?;

				let input: Option<Value> = params.get("input").cloned();

				let workspace = workspace
					.setup((&package).into())
					.build()
					.map_err(|err| err.to_string())?;

				let compiler = {
					let base = compiler.clone().workspace(workspace);

					match input {
						None => base,
						Some(input) => base.prop(Box::new(Input(input))),
					}
				};

				let rendered = package
					.compile_with(compiler)
					.map_err(|err| err.to_string())?;

				Ok(rendered)
			},
		);

		Output::Callback(Function { params, handler })
	}
}
