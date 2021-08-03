use lazy_static::lazy_static;
use regex::Regex;
use serde_json::Value;
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

pub fn find(json: &Value, filter: &Filter) -> Result<Vec<(PathBuf, Value)>> {
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
				objects.push((base, json.to_owned()));
			}
		} else {
			match json {
				Value::Object(map) => {
					let mut members: Vec<(PathBuf, &Value)> = Vec::with_capacity(map.len());

					for (k, v) in map {
						if !is_valid_path(k) {
							return Err(Error::Invalid);
						} else {
							let mut path = base.clone();
							path.push(k);

							members.push((path, v))
						}
					}

					walker.push(Box::new(members.into_iter()));
				}
				_ => return Err(Error::Invalid),
			}
		}
	}

	Ok(objects)
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

	validator.validate(obj).is_strictly_valid()
}

fn is_valid_path(path: &str) -> bool {
	lazy_static! {
		static ref PATTERN: Regex =
			Regex::new(r"(?i)^[a-z0-9]$|^[a-z0-9][a-z0-9-]*[a-z0-9]$").unwrap();
	}

	PATTERN.is_match(path)
}
