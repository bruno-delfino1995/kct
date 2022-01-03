mod file;
mod resolvers;
mod subpackage;

use self::resolvers::*;
use crate::error::{Error, Result};
use crate::Package;
use jrsonnet_evaluator::{
	error::Error as JrError,
	error::LocError,
	trace::{ExplainingFormat, PathResolver},
	EvaluationState, ManifestFormat, Val,
};
use serde_json::{Map, Value};
use std::collections::HashMap;
use std::convert::From;
use std::path::{Path, PathBuf};
use std::rc::Rc;

pub const LIB_CODE: &str = include_str!("lib.libsonnet");
pub const VARS_PREFIX: &str = "kct.io";
pub const FILES_PARAM: &str = "files";
pub const INCLUDE_PARAM: &str = "include";
pub const PACKAGE_PARAM: &str = "package";
pub const RELEASE_PARAM: &str = "release";
pub const INPUT_PARAM: &str = "input";
pub const TEMPLATES_FOLDER: &str = "files";
pub const VENDOR_FOLDER: &str = "vendor";
pub const LIB_FOLDER: &str = "lib";

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

		let variables = self.create_ext_vars(&input);
		for (name, value) in variables.iter() {
			let name = format!("{}/{}", VARS_PREFIX, name);
			self.state.add_ext_var(name.into(), value.clone())
		}

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

	fn create_ext_vars(&self, input: &Value) -> HashMap<String, Val> {
		let files = file::create_function(&self.package, input);
		let include = subpackage::create_function(self);
		let input = Val::from(input);
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

		vec![
			(PACKAGE_PARAM, package),
			(RELEASE_PARAM, release),
			(INPUT_PARAM, input),
			(INCLUDE_PARAM, include),
			(FILES_PARAM, files),
		]
		.into_iter()
		.map(|(k, v)| (String::from(k), v))
		.collect()
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

	let sdk_resolver = Box::new(StaticImportResolver {
		path: PathBuf::from(VARS_PREFIX),
		contents: String::from(LIB_CODE),
	});

	let relative_resolver = Box::new(RelativeImportResolver);

	let lib_resolver = Box::new(LibImportResolver {
		library_paths: vec![vendor, lib],
	});

	let resolver = AggregatedImportResolver::default()
		.push(sdk_resolver)
		.push(relative_resolver)
		.push(lib_resolver);

	state.set_import_resolver(Box::new(resolver));

	state.set_manifest_format(ManifestFormat::Json(0));

	state
}
