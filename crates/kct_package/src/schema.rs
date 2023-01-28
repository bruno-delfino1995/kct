use crate::error::{Error, Result};

use std::convert::TryFrom;
use std::path::PathBuf;
use std::rc::Rc;

use kct_compiler::extension::{Extension, Plugin};
use kct_compiler::Runtime;
use kct_helper::io;
use serde_json::Value;
use url::Url;
use valico::json_schema::Scope;

#[derive(Debug)]
pub struct Schema {
	schema: Rc<Value>,
	scope: Scope,
	id: Url,
}

impl Clone for Schema {
	fn clone(&self) -> Self {
		Schema::try_from(self.schema.as_ref()).unwrap()
	}
}

impl TryFrom<&Value> for Schema {
	type Error = Error;

	fn try_from(schema: &Value) -> Result<Self> {
		let mut scope = Scope::new();
		let id = scope
			.compile(schema.clone(), false)
			.map_err(|_err| Error::InvalidSchema)?;
		let schema = Rc::new(schema.to_owned());

		Ok(Schema { schema, scope, id })
	}
}

impl TryFrom<PathBuf> for Schema {
	type Error = Error;

	fn try_from(path: PathBuf) -> Result<Self> {
		match io::from_file(&path) {
			Ok(contents) => {
				let schema: Value =
					serde_json::from_str(&contents).map_err(|_err| Error::InvalidSchema)?;

				let schema = Self::try_from(&schema)?;

				Ok(schema)
			}
			_ => Err(Error::InvalidSchema),
		}
	}
}

impl Schema {
	pub fn validate(&self, value: &Value) -> bool {
		let schema = self.scope.resolve(&self.id).unwrap();

		schema.validate(value).is_strictly_valid()
	}
}

impl Extension for Schema {
	fn plug(&self, _: Runtime) -> Plugin {
		let schema = self.clone();
		let predicate = move |input: &Value| -> std::result::Result<(), String> {
			if !input.is_object() {
				return Err("input is not an object".to_string());
			}

			if schema.validate(input) {
				Ok(())
			} else {
				Err("input doesn't match your schema".to_string())
			}
		};

		Plugin::Verify(Rc::new(predicate))
	}
}
