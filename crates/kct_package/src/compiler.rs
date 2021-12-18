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

pub const NAME_PARAM: &str = "name";
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

#[derive(Clone)]
pub struct Compiler {
	pub package: Package,
	pub release: Rc<Option<Release>>,
	pub vendor: Rc<PathBuf>,
	state: EvaluationState,
}

impl Compiler {
	pub fn new(package: Package) -> Self {
		let release = Rc::new(None);
		let vendor = {
			let mut path = package.root.clone();
			path.push(VENDOR_FOLDER);

			Rc::new(path)
		};
		let state = create_state(&package.root, &vendor);

		Compiler {
			package,
			release,
			vendor,
			state,
		}
	}

	pub fn with_release(mut self, release: Option<Release>) -> Self {
		self.release = Rc::new(release);

		self
	}

	pub fn fork(&self, package: Package) -> Self {
		let state = create_state(&package.root, &self.vendor);

		Compiler {
			state,
			package,
			vendor: Rc::clone(&self.vendor),
			release: Rc::clone(&self.release),
		}
	}

	pub fn compile(self, input: Option<Value>) -> Result<Value> {
		let render_issue = |err: LocError| {
			let message = match err.error() {
				JrError::ImportSyntaxError { path, .. } => {
					format!("syntax error at {}", path.display())
				}
				err => err.to_string(),
			};

			Error::RenderIssue(message)
		};

		self.package.validate_input(&input)?;

		let input = input.unwrap_or(Value::Null);

		self.state
			.settings_mut()
			.globals
			.insert(GLOBAL_VARIABLE.into(), self.create_global(&input));

		let parsed = self
			.state
			.evaluate_file_raw(&self.package.main)
			.map_err(render_issue)?;

		let rendered = self
			.state
			.manifest(parsed)
			.map_err(render_issue)?
			.to_string();

		let json = serde_json::from_str(&rendered).map_err(|_err| Error::InvalidOutput)?;

		Ok(json)
	}

	fn create_global(&self, input: &Value) -> Val {
		let files = file::create_function(&self.package, input);
		let include = subpackage::create_function(self);
		let input = Val::from(input);
		let name = {
			let name = match self.release.as_ref() {
				Some(release) => format!("{}-{}", release.name, self.package.spec.name),
				None => self.package.spec.name.clone(),
			};

			let value = Value::String(name);

			Val::from(&value)
		};
		let package = {
			let mut map = Map::<String, Value>::new();
			map.insert(
				String::from("name"),
				Value::String(self.package.spec.name.clone()),
			);
			map.insert(
				String::from("version"),
				Value::String(self.package.spec.version.to_string()),
			);

			let value = Value::Object(map);

			Val::from(&value)
		};
		let release = match self.release.as_ref() {
			None => Val::Null,
			Some(release) => {
				let mut map = Map::<String, Value>::new();
				map.insert(String::from("name"), Value::String(release.name.clone()));

				let value = Value::Object(map);

				Val::from(&value)
			}
		};

		let pairs = vec![
			(NAME_PARAM, name),
			(PACKAGE_PARAM, package),
			(RELEASE_PARAM, release),
			(INPUT_PARAM, input),
			(INCLUDE_PARAM, include),
			(FILES_PARAM, files),
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

fn create_state(root: &Path, vendor: &Path) -> EvaluationState {
	let root = root.to_path_buf();
	let state = EvaluationState::default();
	let resolver = PathResolver::Absolute;
	state.set_trace_format(Box::new(ExplainingFormat { resolver }));

	state.with_stdlib();

	let vendor = vendor.to_path_buf();

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
