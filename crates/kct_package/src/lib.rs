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
use std::path::PathBuf;
use tempfile::TempDir;

const SCHEMA_FILE: &str = "values.schema.json";
const SPEC_FILE: &str = "kcp.json";
const VALUES_FILE: &str = "values.json";
const MAIN_FILE: &str = "templates/main.jsonnet";

#[derive(Debug)]
pub struct Package {
	pub root: PathBuf,
	pub main: PathBuf,
	pub spec: Spec,
	pub schema: Option<Schema>,
	pub values: Option<Value>,
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

		let spec = {
			let mut path = root.clone();
			path.push(SPEC_FILE);

			if path.exists() {
				Spec::from_path(path)?
			} else {
				return Err(Error::NoSpec);
			}
		};

		let schema = {
			let mut path = root.clone();
			path.push(SCHEMA_FILE);

			if path.exists() {
				Some(Schema::from_path(path)?)
			} else {
				None
			}
		};

		let values = {
			let mut path = root.clone();
			path.push(VALUES_FILE);

			if path.exists() {
				let contents = io::from_file(&path).map_err(|_err| Error::InvalidValues)?;
				Some(serde_json::from_str(&contents).map_err(|_err| Error::InvalidValues)?)
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

		validate_values(&schema, &values)?;

		Ok(Package {
			root,
			main,
			spec,
			schema,
			values,
			brownfield,
		})
	}
}

/// Methods
impl Package {
	pub fn archive(self, dest: &PathBuf) -> std::result::Result<PathBuf, String> {
		let name = format!("{}_{}", self.spec.name, self.spec.version);
		archive::archive(&name, &self.root, dest)
	}

	pub fn compile(self, values: Option<Value>, release: Option<Release>) -> Result<Value> {
		let values = match (&self.values, &values) {
			(Some(defaults), Some(values)) => {
				let mut merged = defaults.to_owned();
				json::merge(&mut merged, values);

				Some(merged)
			}
			(None, Some(values)) => Some(values.to_owned()),
			(Some(defaults), None) => Some(defaults.to_owned()),
			_ => None,
		};

		validate_values(&self.schema, &values)?;

		compile::compile(self, values.unwrap_or(Value::Null), release)
	}
}

fn validate_values(schema: &Option<Schema>, values: &Option<Value>) -> Result<()> {
	let (schema, values) = match (&schema, &values) {
		(None, None) => return Ok(()),
		(None, Some(_)) => return Err(Error::NoSchema),
		(Some(_), None) => return Err(Error::NoValues),
		(Some(schema), Some(value)) => (schema, value),
	};

	if values.is_object() && schema.validate(&values) {
		Ok(())
	} else {
		Err(Error::InvalidValues)
	}
}
