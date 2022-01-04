pub mod extension;
mod release;
mod resolvers;

pub use self::release::Release;
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
use serde_json::Value;
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

pub trait Callback: Fn(&Compiler) -> NativeCallback {}
impl<T: Fn(&Compiler) -> NativeCallback> Callback for T {}

pub trait Validator: Fn(&Compiler) -> bool {}
impl<T: Fn(&Compiler) -> bool> Validator for T {}

#[derive(Clone, Default)]
pub struct Compiler {
	root: Rc<PathBuf>,
	vendor: Rc<PathBuf>,
	entrypoint: PathBuf,
	properties: HashMap<Property, Rc<Value>>,
	extensions: HashMap<Extension, Rc<Box<dyn Callback>>>,
	validators: Vec<Rc<Box<dyn Validator>>>,
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

	pub fn prop<V: Into<Value>>(mut self, key: Property, value: Option<V>) -> Self {
		match value {
			None => self,
			Some(v) => {
				self.properties.insert(key, Rc::new(v.into()));

				self
			}
		}
	}

	pub fn extension<F: 'static + Callback>(mut self, key: Extension, generator: F) -> Self {
		self.extensions.insert(key, Rc::new(Box::new(generator)));

		self
	}

	pub fn validator<F: 'static + Validator>(mut self, validator: F) -> Self {
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
			state.add_ext_var(name.into(), value);
		}

		let parsed = state
			.evaluate_file_raw(&self.entrypoint)
			.map_err(render_issue)?;

		let rendered = state.manifest(parsed).map_err(render_issue)?.to_string();

		let json = serde_json::from_str(&rendered).map_err(|_err| Error::InvalidOutput)?;

		Ok(json)
	}

	fn create_ext_vars(&self) -> HashMap<String, Val> {
		let from_prop = |p: Property, n: &str| -> (String, Val) {
			let default = Val::Null;
			let value = self.properties.get(&p);

			let val = value.map(|v| Val::from(&(**v))).unwrap_or(default);

			(String::from(n), val)
		};

		let from_ext = |e: Extension, n: &str| -> (String, Val) {
			let val = match self.extensions.get(&e) {
				None => Val::Null,
				Some(func) => {
					let ext: Rc<FuncVal> =
						Rc::new(FuncVal::NativeExt(n.into(), Rc::new(func(self))));

					Val::Func(ext)
				}
			};

			(String::from(n), val)
		};

		vec![
			from_prop(Property::Package, PACKAGE_PARAM),
			from_prop(Property::Release, RELEASE_PARAM),
			from_prop(Property::Input, INPUT_PARAM),
			from_ext(Extension::Include, INCLUDE_PARAM),
			from_ext(Extension::File, FILES_PARAM),
		]
		.into_iter()
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

#[derive(Clone)]
pub struct Compilation {
	pub root: Rc<PathBuf>,
	pub vendor: Rc<PathBuf>,
	pub package: Option<Rc<Value>>,
	pub input: Option<Rc<Value>>,
	pub release: Option<Rc<Value>>,
}

impl From<&Compiler> for Compilation {
	fn from(compiler: &Compiler) -> Self {
		let root = Rc::clone(&compiler.root);
		let vendor = Rc::clone(&compiler.vendor);

		let package = compiler.properties.get(&Property::Package).map(Rc::clone);
		let input = compiler.properties.get(&Property::Input).map(Rc::clone);
		let release = compiler.properties.get(&Property::Release).map(Rc::clone);

		Compilation {
			root,
			vendor,
			package,
			input,
			release,
		}
	}
}
