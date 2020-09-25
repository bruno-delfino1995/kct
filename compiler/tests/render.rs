mod fixtures;

use compiler::{package::schema::Schema, package::spec::Spec, render::error::Error, Package};
use fixtures::Fixture;
use serde_json::Value;

fn create_package_with(templates: &[&str], schema: Option<&str>) -> Package {
	let root = Fixture::dir(&templates);

	let mut main = root.clone();
	main.push(templates[0]);
	let spec = Spec {
		main,
		name: String::from("fixture"),
	};

	let schema = schema
		.map(serde_json::from_str)
		.map(Result::unwrap)
		.map(Schema::new)
		.map(Result::unwrap);

	Package { root, spec, schema }
}

const MINIMAL_SCHEMA: &str = r#"{
	"$schema": "http://json-schema.org/schema#",
	"type": "object",
	"additionalProperties": false,
	"required": ["secret"],
	"properties": {
		"secret": {
			"type": "string"
		}
	}
}"#;

const MINIMAL_VALUES: &str = r#"{
	"secret": "ultra secret value"
}"#;

#[test]
fn requests_values() {
	let package = create_package_with(&["valid.jsonnet"], Some(MINIMAL_SCHEMA));

	let rendered = compiler::render(&package, None);
	assert_eq!(rendered.unwrap_err(), Error::NoValues);
}

#[test]
fn request_schema() {
	let package = create_package_with(&["valid.jsonnet"], None);
	let values = serde_json::from_str(MINIMAL_VALUES).unwrap();

	let rendered = compiler::render(&package, values);
	assert_eq!(rendered.unwrap_err(), Error::NoSchema);
}

#[test]
fn renders_with_null() {
	let package = create_package_with(&["valid.jsonnet"], None);
	let rendered = compiler::render(&package, None);

	let json = r#"{ "values": null }"#;
	let result: Value = serde_json::from_str(json).unwrap();
	assert_eq!(rendered.unwrap(), result);
}

#[test]
fn renders_with_value() {
	let package = create_package_with(&["valid.jsonnet"], Some(MINIMAL_SCHEMA));
	let value = serde_json::from_str(MINIMAL_VALUES).unwrap();

	let rendered = compiler::render(&package, Some(value));

	let json = format!(r#"{{ "values": {0} }}"#, MINIMAL_VALUES);
	let result: Value = serde_json::from_str(&json).unwrap();
	assert_eq!(rendered.unwrap(), result);
}

#[test]
fn expects_tla() {
	let package = create_package_with(&["plain.jsonnet"], None);

	let rendered = compiler::render(&package, None).unwrap_err();

	match rendered {
		Error::RenderIssue(_) => (),
		_ => panic!("It should be a render issue!"),
	}
}

#[test]
fn expects_tla_with_values_param() {
	let package = create_package_with(&["no-param.jsonnet"], None);

	let rendered = compiler::render(&package, None).unwrap_err();

	match rendered {
		Error::RenderIssue(_) => (),
		_ => panic!("It should be a render issue!"),
	}
}

#[test]
fn renders_imports() {
	let package = create_package_with(&["import.jsonnet", "valid.jsonnet"], Some(MINIMAL_SCHEMA));
	let value = serde_json::from_str(MINIMAL_VALUES).unwrap();

	let rendered = compiler::render(&package, Some(value));

	let json = format!(r#"{{ "imported": {{ "values": {0} }} }}"#, MINIMAL_VALUES);
	let result: Value = serde_json::from_str(&json).unwrap();
	assert_eq!(rendered.unwrap(), result);
}
