use crate::error::{Error, Result};
use crate::Package;
use globwalk::{DirEntry, GlobWalkerBuilder};
use jrsonnet_evaluator::{
	error::Error as JrError,
	error::LocError,
	native::NativeCallback,
	trace::{ExplainingFormat, PathResolver},
	EvaluationState, FileImportResolver, FuncVal, Val,
};
use jrsonnet_parser::{Param, ParamsDesc};
use serde_json::{Map, Value};
use std::fs;
use std::path::PathBuf;
use std::rc::Rc;
use tera::{Context, Tera};

const VALUES_PARAM: &str = "values";
const FILES_PARAM: &str = "files";
const TEMPLATES_FOLDER: &str = "files";

pub fn compile(pkg: Package, values: Value) -> Result<Value> {
	let state = create_state(&pkg);

	let render_issue = |err: LocError| Error::RenderIssue(format!("{}", err.error()));

	state.add_tla(FILES_PARAM.into(), create_files_func(&pkg, &values));

	state
		.add_tla_code(VALUES_PARAM.into(), values.to_string().into())
		.map_err(render_issue)?;

	let parsed = state
		.evaluate_file_raw(&pkg.spec.main)
		.map_err(render_issue)?;

	let parsed = match parsed {
		Val::Func(_) => parsed,
		_ => return Err(Error::RenderIssue(String::from("Template is not a TLA"))),
	};

	let wrapped = state.with_tla(parsed).map_err(render_issue)?;

	let rendered = state.manifest(wrapped).map_err(render_issue)?.to_string();

	let json = serde_json::from_str(&rendered).map_err(|_err| Error::InvalidOutput)?;

	Ok(json)
}

fn create_state(pkg: &Package) -> EvaluationState {
	let state = EvaluationState::default();
	let resolver = PathResolver::Absolute;
	state.set_trace_format(Box::new(ExplainingFormat { resolver }));

	state.with_stdlib();

	state.set_import_resolver(Box::new(FileImportResolver {
		library_paths: vec![pkg.root.clone()],
	}));

	state
}

fn create_files_func(pkg: &Package, values: &Value) -> Val {
	let params = ParamsDesc(Rc::new(vec![Param("name".into(), None)]));

	let root = pkg.root.clone();
	let values = values.clone();
	let render = move |params: &[Val]| -> std::result::Result<Val, LocError> {
		let name = params.get(0).unwrap();
		let file = match name {
			Val::Str(name) => name,
			_ => {
				return Err(LocError::new(JrError::AssertionFailed(
					"name should be a string".into(),
				)))
			}
		};

		let compiled = compile_template(&root, file, &values)
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
	let ext: Rc<FuncVal> = FuncVal::NativeExt("files".into(), func.into()).into();

	Val::Func(ext)
}

fn compile_template(
	root: &PathBuf,
	glob: &str,
	values: &Value,
) -> std::result::Result<Vec<String>, String> {
	let mut templates_dir = root.clone();
	templates_dir.push(TEMPLATES_FOLDER);

	if !templates_dir.exists() {
		return Err(String::from("No files folder to search for templates"));
	}

	let globwalker = GlobWalkerBuilder::new(templates_dir, glob)
		.build()
		.map_err(|err| format!("Invalid glob provided ({}): {}", glob, err))?;

	let dirs: Vec<DirEntry> = globwalker
		.collect::<std::result::Result<_, _>>()
		.map_err(|err| format!("Unable to resolve globs: {}", err))?;

	let contents: Vec<String> = dirs
		.into_iter()
		.map(DirEntry::into_path)
		.map(fs::read_to_string)
		.collect::<std::result::Result<_, _>>()
		.map_err(|err| format!("Unable to read templates: {}", err))?;

	let context = match values {
		Value::Null => Context::from_serialize(Value::Object(Map::new())).unwrap(),
		_ => Context::from_serialize(values).unwrap(),
	};

	let compiled: Vec<String> = contents
		.into_iter()
		.map(|content| Tera::one_off(&content, &context, true))
		.collect::<std::result::Result<_, _>>()
		.map_err(|err| format!("Unable to compile templates: {}", err))?;

	Ok(compiled)
}
