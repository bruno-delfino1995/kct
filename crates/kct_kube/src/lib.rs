use serde_json::{Map, Value};
use std::path::{Path, PathBuf};
use thiserror::Error;
use valico::json_schema::Scope;

#[derive(Error, PartialEq, Debug)]
pub enum Error {
	#[error("The rendered json is invalid")]
	Invalid,
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Default)]
pub struct Filter {
	pub only: Vec<PathBuf>,
	pub except: Vec<PathBuf>,
}

impl Filter {
	fn pass(&self, path: &Path) -> bool {
		let allow = self.only.iter().any(|allow| path.starts_with(allow));

		let disallow = self
			.except
			.iter()
			.any(|disallow| path.starts_with(disallow));

		(allow || self.only.is_empty()) && !disallow
	}
}

pub fn find(json: &Value, filter: &Filter) -> Result<Vec<Value>> {
	let mut objects = vec![];
	let mut walker: Vec<Box<dyn Iterator<Item = (PathBuf, &Value)>>> =
		vec![Box::new(vec![(PathBuf::from("/"), json)].into_iter())];

	while let Some(curr) = walker.last_mut() {
		let (base, json) = match curr.next() {
			Some(val) => val,
			None => {
				walker.pop();
				continue;
			}
		};

		if is_object(json) {
			if filter.pass(&base) {
				objects.push(json.to_owned());
			}
		} else {
			match json {
				Value::Object(map) => {
					walker.push(Box::new(map.into_iter().map(move |(k, v)| {
						let mut path = base.clone();
						path.push(k);

						(path, v)
					})));
				}
				_ => return Err(Error::Invalid),
			}
		}
	}

	Ok(objects)
}

pub fn glue(obj: &[Value]) -> Value {
	if obj.len() == 1 {
		obj[0].to_owned()
	} else {
		let object = {
			let mut map = Map::<String, Value>::new();
			map.insert(
				String::from("apiVersion"),
				Value::String(String::from("v1")),
			);
			map.insert(String::from("kind"), Value::String(String::from("List")));
			map.insert(String::from("items"), Value::Array(obj.to_owned()));
			map
		};

		Value::Object(object)
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
