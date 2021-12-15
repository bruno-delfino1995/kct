use super::{FILES_PARAM, TEMPLATES_FOLDER};
use crate::Package;
use globwalk::{DirEntry, GlobWalkerBuilder};
use jrsonnet_evaluator::{
	error::Error as JrError, error::LocError, native::NativeCallback, FuncVal, Val,
};
use jrsonnet_parser::{Param, ParamsDesc};
use serde_json::{Map, Value};
use std::fs;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use tera::{Context, Tera};

pub fn create_function(pkg: &Package, input: &Value) -> Val {
	let params = ParamsDesc(Rc::new(vec![Param("name".into(), None)]));

	let root = pkg.root.clone();
	let input = input.clone();
	let render = move |_caller, params: &[Val]| -> std::result::Result<Val, LocError> {
		let name = params.get(0).unwrap();
		let file = match name {
			Val::Str(name) => name,
			_ => {
				return Err(LocError::new(JrError::AssertionFailed(
					"name should be a string".into(),
				)))
			}
		};

		let compiled = compile_template(&root, file, &input)
			.map_err(|err| LocError::new(JrError::RuntimeError(err.into())))?;

		if compiled.is_empty() {
			Err(LocError::new(JrError::RuntimeError(
				format!("No template found for glob {}", file).into(),
			)))
		} else if compiled.len() == 1 {
			Ok(Val::Str(compiled.into_iter().next().unwrap().into()))
		} else {
			Ok(Val::Arr(
				compiled
					.into_iter()
					.map(|comp| Val::Str(comp.into()))
					.collect::<Vec<Val>>()
					.into(),
			))
		}
	};

	let func = NativeCallback::new(params, render);
	let ext: Rc<FuncVal> = FuncVal::NativeExt(FILES_PARAM.into(), func.into()).into();

	Val::Func(ext)
}

fn compile_template(
	root: &Path,
	glob: &str,
	input: &Value,
) -> std::result::Result<Vec<String>, String> {
	let mut templates_dir = root.to_path_buf();
	templates_dir.push(TEMPLATES_FOLDER);

	if !templates_dir.exists() {
		return Err(String::from("No files folder to search for templates"));
	}

	let globwalker = GlobWalkerBuilder::new(templates_dir, glob)
		.build()
		.map_err(|err| format!("Invalid glob provided ({}): {}", glob, err))?;

	let entries: Vec<DirEntry> = globwalker
		.collect::<std::result::Result<_, _>>()
		.map_err(|err| format!("Unable to resolve globs: {}", err))?;

	let mut paths: Vec<PathBuf> = entries.into_iter().map(DirEntry::into_path).collect();

	paths.sort();

	let contents: Vec<String> = paths
		.into_iter()
		.map(fs::read_to_string)
		.collect::<std::result::Result<_, _>>()
		.map_err(|err| format!("Unable to read templates: {}", err))?;

	let context = match input {
		Value::Null => Context::from_serialize(Value::Object(Map::new())).unwrap(),
		_ => Context::from_serialize(input).unwrap(),
	};

	let compiled: Vec<String> = contents
		.into_iter()
		.map(|content| Tera::one_off(&content, &context, true))
		.collect::<std::result::Result<_, _>>()
		.map_err(|err| format!("Unable to compile templates: {}", err))?;

	Ok(compiled)
}
