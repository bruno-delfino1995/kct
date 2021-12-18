use super::Compiler;
use super::INCLUDE_PARAM;
use crate::Package;
use jrsonnet_evaluator::{
	error::Error as JrError, error::LocError, native::NativeCallback, FuncVal, Val,
};
use jrsonnet_parser::{Param, ParamsDesc};
use serde_json::Value;
use std::{convert::TryFrom, rc::Rc};

pub fn create_function(compiler: &Compiler) -> Val {
	let params = ParamsDesc(Rc::new(vec![
		Param("name".into(), None),
		Param("input".into(), None),
	]));

	let root = compiler.vendor.clone();
	let subcompiler = compiler.clone();
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

		let root = root.join(&package.to_string());
		let package = Package::try_from(root)
			.map_err(|err| LocError::new(JrError::RuntimeError(err.to_string().into())))?;

		let input: Option<Value> = params
			.get(1)
			.map(|val| val.to_string().unwrap())
			.map(|val| serde_json::from_str(&val).unwrap());

		let rendered = subcompiler
			.fork(package)
			.compile(input)
			.map_err(|err| LocError::new(JrError::RuntimeError(err.to_string().into())))?;

		Ok(Val::from(&rendered))
	};

	let func = NativeCallback::new(params, render);
	let ext: Rc<FuncVal> = FuncVal::NativeExt(INCLUDE_PARAM.into(), func.into()).into();

	Val::Func(ext)
}
