mod error;
mod property;
mod schema;
mod spec;

pub use crate::error::Error;

use crate::error::Result;
use crate::property::{File, Include};
use crate::schema::Schema;
use crate::spec::Spec;

use std::convert::TryFrom;
use std::path::{Path, PathBuf};

use kct_compiler::property::{Name, Prop};
use kct_compiler::{Compiler, Release, Target, TargetBuilder};
use kct_compiler::{Error as CError, Input};
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

		match (&schema, &example) {
			(None, Some(_)) => return Err(Error::NoSchema),
			(Some(_), None) => return Err(Error::NoExample),
			_ => (),
		};

		let package = Package {
			root,
			main,
			spec,
			schema,
			example,
		};

		Ok(package)
	}
}

impl Package {
	pub fn compile(
		self,
		input: Option<Value>,
		release: Option<Release>,
	) -> std::result::Result<Value, CError> {
		let target = (&self).into();
		let input = input.map(|v| (&Input(v)).into());

		let compiler = Compiler::bootstrap(&self.root)
			.with_release(release)
			.with_target(target)
			.with_static_prop(input);

		self.compile_with(compiler)
	}

	pub fn compile_with(self, compiler: Compiler) -> std::result::Result<Value, CError> {
		let compiler = self.augment(compiler);

		compiler.compile()
	}

	fn augment(self, compiler: Compiler) -> Compiler {
		let mut compiler = compiler
			.with_static_prop(Some((&self).into()))
			.with_dynamic_prop(Some(Box::new(File)))
			.with_dynamic_prop(Some(Box::new(Include)));

		compiler = match self.schema {
			Some(schema) => compiler.with_check(schema.into()),
			None => compiler,
		};

		compiler
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

impl From<&Package> for Prop {
	fn from(val: &Package) -> Self {
		Prop::primitive(Name::Package, val.into())
	}
}
