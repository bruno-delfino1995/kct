#![feature(move_ref_pattern)]

pub mod error;
pub mod schema;
pub mod spec;

mod archive;
mod compile;

use self::error::{Error, Result};
use self::schema::Schema;
use self::spec::Spec;
pub use compile::Release;
use serde_json::Value;
use std::path::PathBuf;
use tempfile::TempDir;

const SCHEMA_FILE: &str = "values.schema.json";
const SPEC_FILE: &str = "kcp.json";

#[derive(Debug)]
pub struct Package {
	pub root: PathBuf,
	pub spec: Spec,
	pub schema: Option<Schema>,
	pub brownfield: Option<TempDir>,
}

/// Associated functions
impl Package {
	pub fn from_path(root: PathBuf) -> Result<Self> {
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

		let mut spec = root.clone();
		spec.push(SPEC_FILE);
		let spec = Spec::from_path(spec)?;

		let mut schema = root.clone();
		schema.push(SCHEMA_FILE);
		let schema = match Schema::from_path(schema) {
			Ok(schema) => Some(schema),
			Err(Error::NoSchema) => None,
			Err(err) => return Err(err),
		};

		Ok(Package {
			root,
			spec,
			schema,
			brownfield,
		})
	}
}

/// Methods
impl Package {
	pub fn archive(self, dest: &PathBuf) -> std::result::Result<PathBuf, String> {
		archive::archive(&self.spec.name, &self.root, dest)
	}

	pub fn compile(self, values: Option<Value>, release: Option<Release>) -> Result<Value> {
		let values = validate_values(&self, values)?;

		compile::compile(self, values, release)
	}
}

fn validate_values(pkg: &Package, values: Option<Value>) -> Result<Value> {
	let (schema, values) = match (&pkg.schema, values) {
		(None, None) => return Ok(Value::Null),
		(None, Some(_)) => return Err(Error::NoSchema),
		(Some(_), None) => return Err(Error::NoValues),
		(Some(schema), Some(value)) => (schema, value),
	};

	if schema.validate(&values) {
		Ok(values)
	} else {
		Err(Error::InvalidValues)
	}
}
