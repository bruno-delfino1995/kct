use kube::{Error, Result};
use serde_json::Value;
use std::iter;

const MINIMAL_OBJECT: &str = r#"{
	"kind": "Deployment",
	"apiVersion": "apps/v1"
}"#;

type Return = Result<Vec<Value>>;

fn find_from(text: &str) -> Return {
	let val: Value = serde_json::from_str(text).unwrap();
	kube::find(&val)
}

fn assert_invalid(err: Return) {
	assert!(err.is_err());
	assert_eq!(err.unwrap_err(), Error::Invalid);
}

fn assert_valid(ok: Return, times: usize) {
	assert!(ok.is_ok());

	let object = serde_json::from_str(MINIMAL_OBJECT).unwrap();
	let objects: Vec<Value> = iter::repeat(object).take(times).collect();
	assert_eq!(ok.unwrap(), objects)
}

#[test]
fn finds_objects() {
	let json = MINIMAL_OBJECT;
	let found = find_from(&json);
	assert_valid(found, 1);

	let json = format!(r#"{{"a":{0}, "b":{0}}}"#, MINIMAL_OBJECT);
	let found = find_from(&json);
	assert_valid(found, 2);

	let json = format!(
		r#"{{"a":{{ "b": {0}, "c": {{ "d": {0}}}, "e": {0}}}}}"#,
		MINIMAL_OBJECT
	);
	let found = find_from(&json);
	assert_valid(found, 3);
}

#[test]
fn disallow_unclear_paths() {
	let json = format!("[{0}, {0}]", MINIMAL_OBJECT);
	let found = find_from(&json);
	assert_invalid(found);

	let json = format!(r#"{{"a":{0}, "b":[{0}, {0}]}}"#, MINIMAL_OBJECT);
	let found = find_from(&json);
	assert_invalid(found);

	let json = format!(r#"{{"a":{{ "b": {0}, "c": [{0}, {0}]}}}}"#, MINIMAL_OBJECT);
	let found = find_from(&json);
	assert_invalid(found);
}

#[test]
fn disallow_primitives() {
	let values = [
		"0",
		"\"object\"",
		"null",
		&format!(r#"{{ "a":1,"b":{0} }}"#, MINIMAL_OBJECT),
		&format!(
			r#"{{ "a":{{ "b":{{ "c":null,"d":{0} }} }} }}"#,
			MINIMAL_OBJECT
		),
		&format!(r#"{{ "a":{{ "b":{0} }}, "c": "str" }}"#, MINIMAL_OBJECT),
	];

	for &json in values.iter() {
		let found = find_from(&json);
		assert_invalid(found);
	}
}
