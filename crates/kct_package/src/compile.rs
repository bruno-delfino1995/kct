mod file;
mod subpackage;

use crate::error::{Error, Result};
use crate::Package;
use jrsonnet_evaluator::{
	error::Error as JrError,
	error::LocError,
	trace::{ExplainingFormat, PathResolver},
	EvaluationState, FileImportResolver, LazyBinding, LazyVal, ObjMember, ObjValue, Val,
};
use jrsonnet_parser::Visibility;
use serde_json::{Map, Value};
use std::collections::HashMap;
use std::rc::Rc;

pub const FILES_PARAM: &str = "files";
pub const INCLUDE_PARAM: &str = "include";
pub const PACKAGE_PARAM: &str = "package";
pub const RELEASE_PARAM: &str = "release";
pub const VALUES_PARAM: &str = "values";
pub const TEMPLATES_FOLDER: &str = "files";
pub const SUBPACKAGES_FOLDER: &str = "kcps";
pub const GLOBAL_VARIABLE: &str = "_";

#[derive(Clone, Debug)]
pub struct Release {
	pub name: String,
}

pub fn compile(pkg: Package, values: Value, release: Option<Release>) -> Result<Value> {
	let state = create_state(&pkg);

	let render_issue = |err: LocError| {
		let message = match err.error() {
			JrError::ImportSyntaxError { path, .. } => {
				format!("syntax error at {}", path.display())
			}
			err => err.to_string(),
		};

		Error::RenderIssue(message)
	};

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
	let files = file::create_function(pkg, values);
	let include = subpackage::create_function(pkg, release);
	let values = Val::from(values);
	let package = {
		let mut map = Map::<String, Value>::new();
		map.insert(String::from("name"), Value::String(pkg.spec.name.clone()));
		map.insert(
			String::from("version"),
			Value::String(pkg.spec.version.to_string()),
		);

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
		(INCLUDE_PARAM, include),
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
