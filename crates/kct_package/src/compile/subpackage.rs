use super::{Release, INCLUDE_PARAM, SUBPACKAGES_FOLDER};
use crate::Package;
use jrsonnet_evaluator::{
	error::Error as JrError, error::LocError, native::NativeCallback, FuncVal, Val,
};
use jrsonnet_parser::{Param, ParamsDesc};
use serde_json::Value;
use std::rc::Rc;

pub fn create_function(pkg: &Package, release: &Option<Release>) -> Val {
	let params = ParamsDesc(Rc::new(vec![
		Param("name".into(), None),
		Param("values".into(), None),
	]));

	let root = pkg.root.clone().join(SUBPACKAGES_FOLDER);
	let release = release.clone();
	let render = move |params: &[Val]| -> std::result::Result<Val, LocError> {
		let name = params.get(0).unwrap();
		let package = match name {
			Val::Str(name) => name,
			_ => {
				return Err(LocError::new(JrError::AssertionFailed(
					"name should be a string".into(),
				)))
			}
		};
		let mut root = root.join(&package.to_string());
		root.set_extension("tgz");

		let package = Package::from_path(root)
			.map_err(|err| LocError::new(JrError::RuntimeError(err.to_string().into())))?;

		let values: Option<Value> = params
			.get(1)
			.map(|val| val.to_string().unwrap())
			.map(|val| serde_json::from_str(&val).unwrap());

		let rendered = package
			.compile(values, release.clone())
			.map_err(|err| LocError::new(JrError::RuntimeError(err.to_string().into())))?;

		Ok(Val::from(&rendered))
	};

	let func = NativeCallback::new(params, render);
	let ext: Rc<FuncVal> = FuncVal::NativeExt(INCLUDE_PARAM.into(), func.into()).into();

	Val::Func(ext)
}
