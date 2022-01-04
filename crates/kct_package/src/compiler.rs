pub mod extension;
mod resolvers;

use self::resolvers::*;
use crate::error::{Error, Result};
use crate::Package;
use jrsonnet_evaluator::native::NativeCallback;
use jrsonnet_evaluator::FuncVal;
use jrsonnet_evaluator::{
	error::Error as JrError,
	error::LocError,
	trace::{ExplainingFormat, PathResolver},
	EvaluationState, ManifestFormat, Val,
};
use serde_json::{Map, Value};
use std::collections::HashMap;
use std::convert::From;
use std::hash::Hash;
use std::path::PathBuf;
use std::rc::Rc;

pub const LIB_CODE: &str = include_str!("lib.libsonnet");
pub const VARS_PREFIX: &str = "kct.io";
pub const FILES_PARAM: &str = "files";
pub const INCLUDE_PARAM: &str = "include";
pub const PACKAGE_PARAM: &str = "package";
pub const RELEASE_PARAM: &str = "release";
pub const INPUT_PARAM: &str = "input";
pub const VENDOR_FOLDER: &str = "vendor";
pub const LIB_FOLDER: &str = "lib";

#[derive(Clone, Debug)]
pub struct Release {
	pub name: String,
}

impl From<Release> for Val {
	fn from(release: Release) -> Self {
		let mut map = Map::<String, Value>::new();
		map.insert(String::from("name"), Value::String(release.name));

		let value = Value::Object(map);

		Val::from(&value)
	}
}

#[derive(Clone, Hash, PartialEq, Eq)]

pub enum Property {
	Package,
	Release,
	Input,
}

#[derive(Clone, Hash, PartialEq, Eq)]

pub enum Extension {
	File,
	Include,
}

pub type Callback = Box<dyn Fn(&Compiler) -> NativeCallback>;
pub type Validator = Box<dyn Fn(&Compiler) -> bool>;

#[derive(Clone, Default)]
pub struct Compiler {
	pub root: Rc<PathBuf>,
	pub vendor: Rc<PathBuf>,
	pub entrypoint: PathBuf,
	pub properties: HashMap<Property, Rc<Val>>,
	pub extensions: HashMap<Extension, Rc<Callback>>,
	pub validators: Vec<Rc<Validator>>,
}

impl Compiler {
	pub fn new(package: &Package) -> Self {
		let root = Rc::new(package.root.clone());
		let vendor = {
			let mut path = package.root.clone();
			path.push(VENDOR_FOLDER);

			Rc::new(path)
		};
		let entrypoint = package.main.clone();

		Compiler {
			root,
			vendor,
			entrypoint,
			..Default::default()
		}
	}

	pub fn fork(&self, package: &Package) -> Self {
		Compiler {
			root: Rc::new(package.root.clone()),
			entrypoint: package.main.clone(),
			..self.clone()
		}
	}

	pub fn prop<V: Into<Val>>(mut self, key: Property, value: Option<V>) -> Self {
		match value {
			None => self,
			Some(v) => {
				self.properties.insert(key, Rc::new(v.into()));

				self
			}
		}
	}

	pub fn extension<F: 'static + Fn(&Compiler) -> NativeCallback>(
		mut self,
		key: Extension,
		generator: F,
	) -> Self {
		self.extensions.insert(key, Rc::new(Box::new(generator)));

		self
	}

	pub fn validator<F: 'static + Fn(&Compiler) -> bool>(mut self, validator: F) -> Self {
		self.validators.push(Rc::new(Box::new(validator)));

		self
	}

	pub fn compile(self) -> Result<Value> {
		let render_issue = |err: LocError| {
			let message = match err.error() {
				JrError::ImportSyntaxError { path, .. } => {
					format!("syntax error at {}", path.display())
				}
				err => err.to_string(),
			};

			Error::RenderIssue(message)
		};

		for validator in self.validators.iter() {
			if !validator(&self) {
				return Err(Error::InvalidInput);
			}
		}

		let state = self.create_state();

		let variables = self.create_ext_vars();
		for (name, value) in variables {
			let name = format!("{}/{}", VARS_PREFIX, name);
			state.add_ext_var(name.into(), (*value).clone());
		}

		let parsed = state
			.evaluate_file_raw(&self.entrypoint)
			.map_err(render_issue)?;

		let rendered = state.manifest(parsed).map_err(render_issue)?.to_string();

		let json = serde_json::from_str(&rendered).map_err(|_err| Error::InvalidOutput)?;

		Ok(json)
	}

	fn create_ext_vars(&self) -> HashMap<String, Rc<Val>> {
		let package = Rc::clone(
			self.properties
				.get(&Property::Package)
				.unwrap_or(&Rc::new(Val::Null)),
		);
		let release = Rc::clone(
			self.properties
				.get(&Property::Release)
				.unwrap_or(&Rc::new(Val::Null)),
		);
		let input = Rc::clone(
			self.properties
				.get(&Property::Input)
				.unwrap_or(&Rc::new(Val::Null)),
		);

		let include = {
			let func = self.extensions.get(&Extension::Include).unwrap();

			let ext: Rc<FuncVal> =
				FuncVal::NativeExt(INCLUDE_PARAM.into(), func(self).into()).into();

			Rc::new(Val::Func(ext))
		};

		let files = {
			let func = self.extensions.get(&Extension::File).unwrap();

			let ext: Rc<FuncVal> = FuncVal::NativeExt(FILES_PARAM.into(), func(self).into()).into();

			Rc::new(Val::Func(ext))
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

	fn create_state(&self) -> EvaluationState {
		let root = (*self.root).clone();
		let state = EvaluationState::default();
		let resolver = PathResolver::Absolute;
		state.set_trace_format(Box::new(ExplainingFormat { resolver }));

		state.with_stdlib();

		let vendor = (*self.vendor).clone();

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
}
