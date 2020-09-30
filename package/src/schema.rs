use super::{Error, Result};
use helper::io::{self, Error as IOError};
use serde_json::Value;
use std::path::PathBuf;
use url::Url;
use valico::json_schema::Scope;

#[derive(Debug)]
pub struct Schema {
	scope: Scope,
	id: Url,
}

/// Associated functions
impl Schema {
	pub fn new(schema: Value) -> Result<Self> {
		let mut scope = Scope::new();
		let id = scope
			.compile(schema, false)
			.map_err(|_err| Error::InvalidSchema)?;

		Ok(Schema { scope, id })
	}

	pub fn from_path(path: PathBuf) -> Result<Self> {
		match io::from_file(&path) {
			Ok(contents) => {
				let schema: Value =
					serde_json::from_str(&contents).map_err(|_err| Error::InvalidSchema)?;

				let schema = Self::new(schema)?;

				Ok(schema)
			}
			Err(IOError::NotFound) => Err(Error::NoSchema),
			_ => Err(Error::InvalidSchema),
		}
	}
}

/// Methods
impl Schema {
	pub fn validate(&self, value: &Value) -> bool {
		let schema = self.scope.resolve(&self.id).unwrap();

		schema.validate(value).is_strictly_valid()
	}
}
