mod schema;
mod spec;

use self::schema::Schema;
use self::spec::Spec;

use crate::compiler::context::ContextBuilder;
use crate::compiler::property::{Name, Output, Property};
use crate::compiler::{Compiler, Runtime};
use crate::compiler::{Workspace, WorkspaceBuilder};
use crate::error::{Error, Result};
use crate::functions::{File, Include};
use crate::input::Input;
use crate::release::Release;

use kct_helper::io;
use serde_json::{Map, Value};
use std::convert::TryFrom;
use std::path::{Path, PathBuf};
use std::rc::Rc;

const SCHEMA_FILE: &str = "schema.json";
const SPEC_FILE: &str = "kcp.json";
const EXAMPLE_FILE: &str = "example.json";
const MAIN_FILE: &str = "templates/main.jsonnet";

#[derive(Debug, Clone)]
pub struct Package {
	pub root: PathBuf,
	pub main: PathBuf,
	pub spec: Spec,
	pub schema: Option<Schema>,
	pub example: Option<Value>,
}

impl TryFrom<&Path> for Package {
	type Error = Error;

	fn try_from(root: &Path) -> Result<Self> {
		let root = PathBuf::from(root);

		let spec = {
			let mut path = root.clone();
			path.push(SPEC_FILE);

			if path.exists() {
				Spec::try_from(path)?
			} else {
				return Err(Error::NoSpec);
			}
		};

		let schema = {
			let mut path = root.clone();
			path.push(SCHEMA_FILE);

			if path.exists() {
				Some(Schema::try_from(path)?)
			} else {
				None
			}
		};

		let example = {
			let mut path = root.clone();
			path.push(EXAMPLE_FILE);

			if path.exists() {
				let value = io::from_file(&path)
					.map_err(|_err| Error::InvalidExample)
					.and_then(|contents| {
						serde_json::from_str(&contents).map_err(|_err| Error::InvalidExample)
					})?;

				Some(value)
			} else {
				None
			}
		};

		let main = {
			let mut path = root.clone();
			path.push(MAIN_FILE);

			if path.exists() {
				path
			} else {
				return Err(Error::NoMain);
			}
		};

		let package = Package {
			root,
			main,
			spec,
			schema,
			example,
		};

		package
			.validate_input(&package.example)
			.map_err(|err| match err {
				Error::InvalidInput => Error::InvalidExample,
				Error::NoInput => Error::NoExample,
				err => err,
			})?;

		Ok(package)
	}
}

/// Methods
impl Package {
	pub fn validate_input(&self, input: &Option<Value>) -> Result<()> {
		let (schema, input) = match (&self.schema, &input) {
			(None, None) => return Ok(()),
			(None, Some(_)) => return Err(Error::NoSchema),
			(Some(_), None) => return Err(Error::NoInput),
			(Some(schema), Some(input)) => (schema, input),
		};

		if input.is_object() && schema.validate(input) {
			Ok(())
		} else {
			Err(Error::InvalidInput)
		}
	}

	pub fn compile(self, input: Option<Value>, release: Option<Release>) -> Result<Value> {
		let workspace = (&self).into();

		let context = {
			let root = self.root.clone();

			let ctx = ContextBuilder::default()
				.release(release)
				.root(root)
				.build()
				.unwrap();

			Rc::new(ctx)
		};

		let mut compiler = Compiler::new(&context, workspace);

		compiler = match input {
			None => compiler,
			Some(input) => compiler.prop(Box::new(Input(input))),
		};

		self.compile_with(compiler)
	}

	pub fn compile_with(self, compiler: Compiler) -> Result<Value> {
		let package = self.clone();
		let validator = move |c: &Compiler| {
			let runtime: Runtime = c.into();

			let input = runtime
				.properties
				.get(&Name::Input)
				.and_then(|v| match v.as_ref() {
					Output::Plain { value, .. } => Some(value.clone()),
					_ => None,
				});

			package.validate_input(&input).is_ok()
		};

		compiler
			.prop(Box::new(self))
			.prop(Box::new(File))
			.prop(Box::new(Include))
			.validator(validator)
			.compile()
	}
}

impl From<&Package> for Value {
	fn from(package: &Package) -> Self {
		let mut map = Map::<String, Value>::new();
		map.insert(
			String::from("name"),
			Value::String(package.spec.name.clone()),
		);
		map.insert(
			String::from("version"),
			Value::String(package.spec.version.to_string()),
		);

		Value::Object(map)
	}
}

impl From<&Package> for Workspace {
	fn from(package: &Package) -> Self {
		let dir = package.root.clone();
		let main = package.main.clone();

		WorkspaceBuilder::default()
			.dir(dir)
			.main(main)
			.build()
			.unwrap()
	}
}

impl Property for Package {
	fn generate(&self, _: Runtime) -> Output {
		Output::Plain {
			name: Name::Package,
			value: self.into(),
		}
	}
}
