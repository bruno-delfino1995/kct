use crate::compiler::Input;
use crate::extension::{Extension, Name};
use crate::{Compiler, Package};
use jrsonnet_evaluator::{error::Error as JrError, error::LocError, native::NativeCallback, Val};
use jrsonnet_parser::{Param, ParamsDesc};
use serde_json::Value;
use std::{convert::TryFrom, rc::Rc};

pub struct Include;

impl Extension for Include {
	fn name(&self) -> Name {
		Name::Include
	}

	fn generate(&self, compiler: &Compiler) -> NativeCallback {
		let params = ParamsDesc(Rc::new(vec![
			Param("name".into(), None),
			Param("input".into(), None),
		]));

		let vendor = compiler.workspace.vendor.to_path_buf();
		let workspace = compiler.workspace.clone();
		let compiler = compiler.clone();
		let render = move |_caller, params: &[Val]| -> std::result::Result<Val, LocError> {
			let name = params.get(0).unwrap();
			let package = match name {
				Val::Str(name) => name,
				_ => {
					return Err(LocError::new(JrError::AssertionFailed(
						"name should be a string".into(),
					)))
				}
			};

			let root = vendor.join(&package.to_string());
			let package = Package::try_from(root)
				.map_err(|err| LocError::new(JrError::RuntimeError(err.to_string().into())))?;

			let input: Option<Value> = params
				.get(1)
				.map(|val| val.to_string().unwrap())
				.map(|val| serde_json::from_str(&val).unwrap());

			let workspace = workspace
				.setup((&package).into())
				.build()
				.map_err(|err| LocError::new(JrError::RuntimeError(err.to_string().into())))?;

			let compiler = {
				let base = compiler.clone().workspace(workspace);

				match input {
					None => base,
					Some(input) => base.prop(Box::new(Input(input))),
				}
			};

			let rendered = package
				.compile_with(compiler)
				.map_err(|err| LocError::new(JrError::RuntimeError(err.to_string().into())))?;

			Ok(Val::from(&rendered))
		};

		NativeCallback::new(params, render)
	}
}
