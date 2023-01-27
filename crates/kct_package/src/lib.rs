mod error;
mod extension;
mod schema;
mod spec;

pub use crate::error::Error;

use crate::error::Result;
use crate::extension::{File, Include};
use crate::schema::Schema;
use crate::spec::Spec;

use std::convert::TryFrom;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use kct_compiler::extension::{Extension, Name, Plugin};
use kct_compiler::ContextBuilder;
use kct_compiler::Error as CError;
use kct_compiler::{Compiler, Input, Release, Runtime, Target, TargetBuilder};
use kct_helper::io;
use serde_json::{Map, Value};

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

	pub fn compile(
		self,
		input: Option<Value>,
		release: Option<Release>,
	) -> std::result::Result<Value, CError> {
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
			Some(input) => compiler.extend(Box::new(Input(input))),
		};

		self.compile_with(compiler)
	}

	pub fn compile_with(self, compiler: Compiler) -> std::result::Result<Value, CError> {
		let package = self.clone();
		let validator = move |c: &Compiler| {
			let runtime: Runtime = c.into();

			let input = runtime
				.plugins
				.get(Name::Input)
				.and_then(|v| match v.as_ref() {
					Plugin::Property { value, .. } => Some(value.clone()),
					_ => None,
				});

			package.validate_input(&input).is_ok()
		};

		compiler
			.extend(Box::new(self))
			.extend(Box::new(File))
			.extend(Box::new(Include))
			.validator(validator)
			.compile()
	}
}

impl From<&Package> for Target {
	fn from(package: &Package) -> Self {
		let dir = package.root.clone();
		let main = package.main.clone();

		TargetBuilder::default()
			.dir(dir)
			.main(main)
			.build()
			.unwrap()
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

impl Extension for Package {
	fn plug(&self, _: Runtime) -> Plugin {
		Plugin::Property {
			name: Name::Package,
			value: self.into(),
		}
	}
}
