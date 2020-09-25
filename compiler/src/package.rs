pub mod error;
pub mod schema;
pub mod spec;

use self::error::{Error, Result};
use self::schema::Schema;
use self::spec::Spec;
use std::path::PathBuf;

const SCHEMA_FILE: &str = "values.schema.json";
const SPEC_FILE: &str = "kcp.json";

pub struct Package {
	pub root: PathBuf,
	pub spec: Spec,
	pub schema: Option<Schema>,
}

impl Package {
	pub fn from_path(root: PathBuf) -> Result<Self> {
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

		Ok(Package { root, spec, schema })
	}
}
