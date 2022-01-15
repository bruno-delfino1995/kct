mod schema;
mod spec;

use self::schema::Schema;
use self::spec::Spec;

use crate::archiver;
use crate::compiler::property::{Name, Output, Property};
use crate::compiler::WorkspaceBuilder;
use crate::compiler::{Compilation, Compiler};
use crate::error::{Error, Result};
use crate::functions::{File, Include};
use crate::input::Input;
use crate::release::Release;

use kct_helper::io;
use serde_json::{Map, Value};
use std::convert::TryFrom;
use std::path::{Path, PathBuf};

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

impl TryFrom<PathBuf> for Package {
	type Error = Error;

	fn try_from(root: PathBuf) -> Result<Self> {
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
	pub fn archive(self, dest: &Path) -> std::result::Result<PathBuf, String> {
		let name = format!("{}_{}", self.spec.name, self.spec.version);
		archiver::archive(&name, &self.root, dest)
	}

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
		let workspace_builder: WorkspaceBuilder = (&self).into();
		let mut compiler = Compiler::try_from(workspace_builder)?;

		compiler = match input {
			None => compiler,
			Some(input) => compiler.prop(Box::new(Input(input))),
		};

		compiler = match release {
			None => compiler,
			Some(release) => compiler.prop(Box::new(release)),
		};

		self.compile_with(compiler)
	}

	pub fn compile_with(self, compiler: Compiler) -> Result<Value> {
		let package = self.clone();
		let validator = move |c: &Compiler| {
			let compilation: Compilation = c.into();

			let input = compilation.input.map(|v| (*v).clone());

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

impl From<&Package> for WorkspaceBuilder {
	fn from(package: &Package) -> Self {
		let root = package.root.clone();
		let entrypoint = package.main.clone();

		WorkspaceBuilder::default()
			.root(root)
			.entrypoint(entrypoint)
	}
}

impl Property for Package {
	fn name(&self) -> Name {
		Name::Package
	}

	fn generate(&self, _: &Compiler) -> Output {
		Output::Plain(self.into())
	}
}
