pub mod extension;
mod release;
mod resolvers;

pub use self::release::Release;
use self::resolvers::*;
use crate::error::{Error, Result};
use derive_builder::Builder;
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
use std::path::{Path, PathBuf};
use std::rc::Rc;

pub const LIB_CODE: &str = include_str!("lib.libsonnet");
pub const VARS_PREFIX: &str = "kct.io";

#[derive(Clone, Hash, PartialEq, Eq)]
pub enum Property {
	Package,
	Release,
	Input,
}

impl Property {
	fn as_str(&self) -> &str {
		use Property::*;

		match self {
			Package => "package",
			Release => "release",
			Input => "input",
		}
	}
}

#[derive(Clone, Hash, PartialEq, Eq)]
pub enum Extension {
	File,
	Include,
}

impl Extension {
	fn as_str(&self) -> &str {
		use Extension::*;

		match self {
			File => "files",
			Include => "include",
		}
	}
}

#[derive(Clone, Default, Builder)]
#[builder(pattern = "owned")]
pub struct Workspace {
	root: PathBuf,
	entrypoint: PathBuf,
	lib: PathBuf,
	vendor: Rc<PathBuf>,
}

impl Workspace {
	fn setup(&self, builder: WorkspaceBuilder) -> WorkspaceBuilder {
		let builder = match builder.vendor {
			None => builder.vendor(Rc::clone(&self.vendor)),
			Some(_) => builder,
		};

		let builder = match builder.lib {
			None => builder.lib(self.lib.clone()),
			Some(_) => builder,
		};

		let builder = match builder.root {
			None => builder.root(self.root.clone()),
			Some(_) => builder,
		};

		

		match builder.entrypoint {
			None => builder.entrypoint(self.entrypoint.clone()),
			Some(_) => builder,
		}
	}
}

pub trait Callback: Fn(&Compiler) -> NativeCallback {}
impl<T: Fn(&Compiler) -> NativeCallback> Callback for T {}

pub trait Validator: Fn(&Compiler) -> bool {}
impl<T: Fn(&Compiler) -> bool> Validator for T {}

#[derive(Clone, Default)]
pub struct Compiler {
	workspace: Workspace,
	properties: HashMap<Property, Rc<Value>>,
	extensions: HashMap<Extension, Rc<Box<dyn Callback>>>,
	validators: Vec<Rc<Box<dyn Validator>>>,
}

impl TryFrom<WorkspaceBuilder> for Compiler {
	type Error = Error;

	fn try_from(builder: WorkspaceBuilder) -> Result<Self> {
		Compiler::setup(builder)
			.build()
			.map_err(|_| Error::InvalidInput)
			.map(|workspace| Compiler {
				workspace,
				..Default::default()
			})
	}
}

impl Compiler {
	pub fn workspace(mut self, workspace: Workspace) -> Self {
		self.workspace = workspace;

		self
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
			.evaluate_file_raw(&self.workspace.entrypoint)
			.map_err(render_issue)?;

		let rendered = state.manifest(parsed).map_err(render_issue)?.to_string();

		let json = serde_json::from_str(&rendered).map_err(|_err| Error::InvalidOutput)?;

		Ok(json)
	}

	fn create_ext_vars(&self) -> HashMap<String, Val> {
		let from_prop = |p: Property| -> (String, Val) {
			let default = Val::Null;
			let name = p.as_str();
			let value = self.properties.get(&p);

			let val = value.map(|v| Val::from(&(**v))).unwrap_or(default);

			(String::from(name), val)
		};

		let from_ext = |e: Extension| -> (String, Val) {
			let name = e.as_str();
			let val = match self.extensions.get(&e) {
				None => Val::Null,
				Some(func) => {
					let ext: Rc<FuncVal> =
						Rc::new(FuncVal::NativeExt(name.into(), Rc::new(func(self))));

					Val::Func(ext)
				}
			};

			(String::from(name), val)
		};

		vec![
			from_prop(Property::Package),
			from_prop(Property::Release),
			from_prop(Property::Input),
			from_ext(Extension::Include),
			from_ext(Extension::File),
		]
		.into_iter()
		.collect()
	}

	fn create_state(&self) -> EvaluationState {
		let state = EvaluationState::default();
		let resolver = PathResolver::Absolute;
		state.set_trace_format(Box::new(ExplainingFormat { resolver }));

		state.with_stdlib();

		let vendor = self.workspace.vendor.to_path_buf();
		let lib = self.workspace.lib.clone();

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

impl Compiler {
	fn setup(builder: WorkspaceBuilder) -> WorkspaceBuilder {
		match builder.root {
			None => builder,
			Some(ref root) => {
				let lib = Self::default_lib(root);
				let vendor = Rc::new(Self::default_vendor(root));

				let builder = match builder.lib {
					None => builder.lib(lib),
					Some(_) => builder,
				};

				

				match builder.vendor {
					None => builder.vendor(vendor),
					Some(_) => builder,
				}
			}
		}
	}

	fn default_vendor(root: &Path) -> PathBuf {
		let mut path = root.to_path_buf();
		path.push("vendor");

		path
	}

	fn default_lib(root: &Path) -> PathBuf {
		let mut path = root.to_path_buf();
		path.push("lib");

		path
	}
}

#[derive(Clone)]
pub struct Compilation {
	pub package: Option<Rc<Value>>,
	pub input: Option<Rc<Value>>,
	pub release: Option<Rc<Value>>,
}

impl From<&Compiler> for Compilation {
	fn from(compiler: &Compiler) -> Self {
		let package = compiler.properties.get(&Property::Package).map(Rc::clone);
		let input = compiler.properties.get(&Property::Input).map(Rc::clone);
		let release = compiler.properties.get(&Property::Release).map(Rc::clone);

		Compilation {
			package,
			input,
			release,
		}
	}
}
