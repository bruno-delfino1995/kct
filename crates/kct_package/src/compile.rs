use crate::error::{Error, Result};
use crate::Package;
use globwalk::{DirEntry, GlobWalkerBuilder};
use jrsonnet_evaluator::{
	error::Error as JrError,
	error::LocError,
	native::NativeCallback,
	trace::{ExplainingFormat, PathResolver},
	EvaluationState, FileImportResolver, FuncVal, LazyBinding, LazyVal, ObjMember, ObjValue, Val,
};
use jrsonnet_parser::{Param, ParamsDesc, Visibility};
use serde_json::{Map, Value};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::rc::Rc;
use tera::{Context, Tera};

const FILES_PARAM: &str = "files";
const PACKAGE_PARAM: &str = "package";
const RELEASE_PARAM: &str = "release";
const VALUES_PARAM: &str = "values";
const TEMPLATES_FOLDER: &str = "files";
const GLOBAL_VARIABLE: &str = "_";

pub struct Release {
	pub name: String,
}

pub fn compile(pkg: Package, values: Value, release: Option<Release>) -> Result<Value> {
	let state = create_state(&pkg);

	let render_issue = |err: LocError| Error::RenderIssue(format!("{}", err.error()));

	state.settings_mut().globals.insert(
		GLOBAL_VARIABLE.into(),
		create_global(&pkg, &values, &release),
	);

	let parsed = state.evaluate_file_raw(&pkg.main).map_err(render_issue)?;

	let rendered = state.manifest(parsed).map_err(render_issue)?.to_string();

	let json = serde_json::from_str(&rendered).map_err(|_err| Error::InvalidOutput)?;

	Ok(json)
}

fn create_state(pkg: &Package) -> EvaluationState {
	let state = EvaluationState::default();
	let resolver = PathResolver::Absolute;
	state.set_trace_format(Box::new(ExplainingFormat { resolver }));

	state.with_stdlib();

	let vendor = {
		let mut path = pkg.root.clone();
		path.push("vendor");

		path
	};

	let lib = {
		let mut path = pkg.root.clone();
		path.push("lib");

		path
	};

	state.set_import_resolver(Box::new(FileImportResolver {
		library_paths: vec![vendor, lib],
	}));

	state
}

fn create_global(pkg: &Package, values: &Value, release: &Option<Release>) -> Val {
	let files = create_files_func(&pkg, values);
	let values = Val::from(values);
	let package = {
		let mut map = Map::<String, Value>::new();
		map.insert(String::from("name"), Value::String(pkg.spec.name.clone()));

		let full_name = match release {
			Some(release) => format!("{}-{}", release.name, pkg.spec.name),
			None => pkg.spec.name.clone(),
		};
		map.insert(String::from("fullName"), Value::String(full_name));

		let value = Value::Object(map);

		Val::from(&value)
	};
	let release = match release {
		None => Val::Null,
		Some(release) => {
			let mut map = Map::<String, Value>::new();
			map.insert(String::from("name"), Value::String(release.name.clone()));

			let value = Value::Object(map);

			Val::from(&value)
		}
	};

	let pairs = vec![
		(FILES_PARAM, files),
		(PACKAGE_PARAM, package),
		(RELEASE_PARAM, release),
		(VALUES_PARAM, values),
	];

	let entries: HashMap<Rc<str>, ObjMember> = pairs
		.into_iter()
		.map(|(k, v)| {
			(
				k.into(),
				ObjMember {
					add: false,
					visibility: Visibility::Normal,
					invoke: LazyBinding::Bound(LazyVal::new_resolved(v)),
					location: None,
				},
			)
		})
		.collect();

	Val::Obj(ObjValue::new(None, Rc::new(entries)))
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
	let ext: Rc<FuncVal> = FuncVal::NativeExt(FILES_PARAM.into(), func.into()).into();

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
