pub mod error;
pub mod schema;
pub mod spec;

mod archive;
mod compile;

use self::error::{Error, Result};
use self::schema::Schema;
use self::spec::Spec;
pub use compile::Release;
use kct_helper::{io, json};
use serde_json::Value;
use std::convert::TryFrom;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

const SCHEMA_FILE: &str = "schema.json";
const SPEC_FILE: &str = "kcp.json";
const DEFAULT_FILE: &str = "default.json";
const MAIN_FILE: &str = "templates/main.jsonnet";

#[derive(Debug)]
pub struct Package {
	pub root: PathBuf,
	pub main: PathBuf,
	pub spec: Spec,
	pub schema: Option<Schema>,
	pub default: Option<Value>,
	pub brownfield: Option<TempDir>,
}

impl TryFrom<PathBuf> for Package {
	type Error = Error;

	fn try_from(root: PathBuf) -> Result<Self> {
		let (root, brownfield) = match root.extension() {
			None => (root, None),
			Some(_) => {
				let brownfield = TempDir::new()
					.expect("Unable to create temporary directory to unpack your KCP");
				let unarchived = PathBuf::from(brownfield.path());

				archive::unarchive(&root, &unarchived).map_err(|_err| Error::InvalidFormat)?;

				(unarchived, Some(brownfield))
			}
		};

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

		let default = {
			let mut path = root.clone();
			path.push(DEFAULT_FILE);

			if path.exists() {
				let contents = io::from_file(&path).map_err(|_err| Error::InvalidInput)?;
				Some(serde_json::from_str(&contents).map_err(|_err| Error::InvalidInput)?)
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

		validate_input(&schema, &default)?;

		Ok(Package {
			root,
			main,
			spec,
			schema,
			default,
			brownfield,
		})
	}
}

/// Methods
impl Package {
	pub fn archive(self, dest: &Path) -> std::result::Result<PathBuf, String> {
		let name = format!("{}_{}", self.spec.name, self.spec.version);
		archive::archive(&name, &self.root, dest)
	}

	pub fn compile(self, input: Option<Value>, release: Option<Release>) -> Result<Value> {
		let input = match (&self.default, &input) {
			(Some(default), Some(input)) => {
				let mut merged = default.to_owned();
				json::merge(&mut merged, input);

				Some(merged)
			}
			(None, Some(input)) => Some(input.to_owned()),
			(Some(default), None) => Some(default.to_owned()),
			_ => None,
		};

		validate_input(&self.schema, &input)?;

		compile::compile(self, input.unwrap_or(Value::Null), release)
	}
}

fn validate_input(schema: &Option<Schema>, input: &Option<Value>) -> Result<()> {
	let (schema, input) = match (&schema, &input) {
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
