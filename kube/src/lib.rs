use serde_json::{Map, Value};
use std::fmt;
use valico::json_schema::Scope;

#[derive(PartialEq, Debug)]
pub enum Error {
	Invalid,
}

impl fmt::Display for Error {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		use self::Error::*;

		match self {
			Invalid => write!(f, "The rendered json is invalid"),
		}
	}
}

pub type Result<T> = std::result::Result<T, Error>;

pub fn find(json: &Value) -> Result<Vec<Value>> {
	if is_object(json) {
		return Ok(vec![json.to_owned()]);
	}

	let mut objects = vec![];
	match json {
		Value::Object(map) => {
			for (_, val) in map.iter() {
				if is_object(&val) {
					objects.push(val.to_owned());
				} else {
					let found = find(&val)?;
					objects.extend_from_slice(&found);
				}
			}

			Ok(objects)
		}
		_ => Err(Error::Invalid),
	}
}

pub fn glue(obj: &[Value]) -> Value {
	if obj.len() == 1 {
		obj[0].to_owned()
	} else {
		let mut map = Map::<String, Value>::new();
		map.insert(
			String::from("apiVersion"),
			Value::String(String::from("v1")),
		);
		map.insert(String::from("kind"), Value::String(String::from("List")));
		map.insert(String::from("items"), Value::Array(obj.to_owned()));

		Value::Object(map)
	}
}

const K8S_OBJECT_SCHEMA: &str = r#"{
	"$schema": "http://json-schema.org/schema#",
	"type": "object",
	"additionalProperties": true,
	"required": ["kind", "apiVersion"],
	"properties": {
		"kind": {
			"type": "string"
		},
		"apiVersion": {
			"type": "string"
		}
	}
}"#;

fn is_object(obj: &Value) -> bool {
	let schema = serde_json::from_str(K8S_OBJECT_SCHEMA).unwrap();

	let mut scope = Scope::new();
	let validator = scope.compile_and_return(schema, false).unwrap();

	validator.validate(&obj).is_strictly_valid()
}
