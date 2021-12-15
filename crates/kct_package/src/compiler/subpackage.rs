use super::Compiler;
use super::{Release, INCLUDE_PARAM};
use crate::Package;
use jrsonnet_evaluator::{
	error::Error as JrError, error::LocError, native::NativeCallback, FuncVal, Val,
};
use jrsonnet_parser::{Param, ParamsDesc};
use serde_json::Value;
use std::{convert::TryFrom, rc::Rc};

pub fn create_function(compiler: &Compiler, release: &Option<Release>) -> Val {
	let params = ParamsDesc(Rc::new(vec![
		Param("name".into(), None),
		Param("input".into(), None),
	]));

	let vendor = compiler.vendor.clone();
	let release = release.clone();
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
		let mut subcompiler = Compiler::new(&root);
		subcompiler.vendor = vendor.clone();

		let package = Package::try_from(root)
			.map_err(|err| LocError::new(JrError::RuntimeError(err.to_string().into())))?;

		let input: Option<Value> = params
			.get(1)
			.map(|val| val.to_string().unwrap())
			.map(|val| serde_json::from_str(&val).unwrap());

		package
			.validate_input(&input)
			.map_err(|err| LocError::new(JrError::RuntimeError(err.to_string().into())))?;

		let rendered = subcompiler
			.compile(package, input.unwrap_or(Value::Null), release.clone())
			.map_err(|err| LocError::new(JrError::RuntimeError(err.to_string().into())))?;

		Ok(Val::from(&rendered))
	};

	let func = NativeCallback::new(params, render);
	let ext: Rc<FuncVal> = FuncVal::NativeExt(INCLUDE_PARAM.into(), func.into()).into();

	Val::Func(ext)
}
