mod file;
mod subpackage;

use crate::error::{Error, Result};
use crate::Package;
use jrsonnet_evaluator::{
	error::Error as JrError,
	error::LocError,
	trace::{ExplainingFormat, PathResolver},
	Context, EvaluationState, FileImportResolver, LazyBinding, LazyVal, ManifestFormat, ObjMember,
	ObjValue, Val,
};
use jrsonnet_interner::IStr;
use jrsonnet_parser::Visibility;
use rustc_hash::FxHashMap;
use serde_json::{Map, Value};
use std::convert::From;
use std::path::{Path, PathBuf};
use std::rc::Rc;

pub const FILES_PARAM: &str = "files";
pub const INCLUDE_PARAM: &str = "include";
pub const PACKAGE_PARAM: &str = "package";
pub const RELEASE_PARAM: &str = "release";
pub const INPUT_PARAM: &str = "input";
pub const TEMPLATES_FOLDER: &str = "files";
pub const VENDOR_FOLDER: &str = "vendor";
pub const LIB_FOLDER: &str = "lib";
pub const GLOBAL_VARIABLE: &str = "_";

#[derive(Clone, Debug)]
pub struct Release {
	pub name: String,
}

pub struct Compiler {
	pub root: PathBuf,
	pub vendor: PathBuf,
	state: EvaluationState,
}

impl Compiler {
	pub fn new(root: &Path) -> Self {
		let root = root.to_path_buf();

		let vendor = {
			let mut path = root.clone();
			path.push(VENDOR_FOLDER);

			path
		};

		let state = create_state(&root);

		Compiler {
			root,
			vendor,
			state,
		}
	}

	pub fn compile(self, pkg: Package, input: Value, release: Option<Release>) -> Result<Value> {
		let render_issue = |err: LocError| {
			let message = match err.error() {
				JrError::ImportSyntaxError { path, .. } => {
					format!("syntax error at {}", path.display())
				}
				err => err.to_string(),
			};

			Error::RenderIssue(message)
		};

		self.state.settings_mut().globals.insert(
			GLOBAL_VARIABLE.into(),
			self.create_global(&pkg, &input, &release),
		);

		let parsed = self
			.state
			.evaluate_file_raw(&pkg.main)
			.map_err(render_issue)?;

		let rendered = self
			.state
			.manifest(parsed)
			.map_err(render_issue)?
			.to_string();

		let json = serde_json::from_str(&rendered).map_err(|_err| Error::InvalidOutput)?;

		Ok(json)
	}

	fn create_global(&self, pkg: &Package, input: &Value, release: &Option<Release>) -> Val {
		let files = file::create_function(pkg, input);
		let include = subpackage::create_function(self, release);
		let input = Val::from(input);
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
			(INPUT_PARAM, input),
			(INCLUDE_PARAM, include),
		];

		let entries: FxHashMap<IStr, ObjMember> = pairs
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

		Val::Obj(ObjValue::new(
			Context::new(),
			None,
			Rc::new(entries),
			Rc::new(vec![]),
		))
	}
}

fn create_state(root: &Path) -> EvaluationState {
	let root = root.to_path_buf();
	let state = EvaluationState::default();
	let resolver = PathResolver::Absolute;
	state.set_trace_format(Box::new(ExplainingFormat { resolver }));

	state.with_stdlib();

	let vendor = {
		let mut path = root.clone();
		path.push(VENDOR_FOLDER);

		path
	};

	let lib = {
		let mut path = root;
		path.push(LIB_FOLDER);

		path
	};

	state.set_import_resolver(Box::new(FileImportResolver {
		library_paths: vec![vendor, lib],
	}));

	state.set_manifest_format(ManifestFormat::Json(0));

	state
}
