use kube::{Error, Filter, Result};
use serde_json::Value;
use std::iter;
use std::path::PathBuf;

const MINIMAL_OBJECT: &str = r#"{
	"kind": "Deployment",
	"apiVersion": "apps/v1"
}"#;

type Return = Result<Vec<Value>>;

fn find_from(text: &str) -> Return {
	let val: Value = serde_json::from_str(text).unwrap();
	kube::find(&val, &Filter::default())
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

mod filter {
	use super::*;

	fn find_within_minimal(filter: &Filter) -> Return {
		let val = serde_json::from_str(MINIMAL_OBJECT).unwrap();
		kube::find(&val, &filter)
	}

	fn find_within_complex(filter: &Filter) -> Return {
		let complex: Value = serde_json::from_str(&format!(
			r#"{{ "a": {{ "b": {0}, "c": {{ "d": {0}, "e": {0} }}, "f": {0} }}, "g": {0} }}"#,
			MINIMAL_OBJECT
		))
		.unwrap();

		kube::find(&complex, filter)
	}

	#[test]
	fn keeps_only_paths() {
		let cases = vec![
			(vec!["/"], 5),
			(vec!["/a/b"], 1),
			(vec!["/a/c"], 2),
			(vec!["/a/b/c"], 0),
			(vec!["/a/c/d", "/a/c"], 2),
			(vec!["/a/c/d", "/a/c/e", "/a/f"], 3),
			(vec!["/a/f", "/g"], 2),
			(vec!["/g", "/a/c"], 3),
		]
		.into_iter()
		.map(|(vec, n)| (vec.iter().map(PathBuf::from).collect(), n));

		for (only, amount) in cases {
			let filter = Filter {
				only,
				except: vec![],
			};

			let found = find_within_complex(&filter);
			assert_valid(found, amount);
		}

		let found = find_within_minimal(&Filter {
			except: vec![],
			only: vec![PathBuf::from("/")],
		});

		assert_valid(found, 1);
	}

	#[test]
	fn discards_disallowed_paths() {
		let cases = vec![
			(vec!["/"], 0),
			(vec!["/a/b"], 4),
			(vec!["/a/c"], 3),
			(vec!["/a/b/c"], 5),
			(vec!["/a/c/d", "/a/c"], 3),
			(vec!["/a/c/d", "/a/c/e", "/a/f"], 2),
			(vec!["/a/f", "/g"], 3),
			(vec!["/g", "/a/c"], 2),
		]
		.into_iter()
		.map(|(vec, n)| (vec.iter().map(PathBuf::from).collect(), n));

		for (except, amount) in cases {
			let filter = Filter {
				except,
				only: vec![],
			};

			let found = find_within_complex(&filter);
			assert_valid(found, amount);
		}

		let found = find_within_minimal(&Filter {
			except: vec![PathBuf::from("/")],
			only: vec![],
		});

		assert_valid(found, 0);
	}

	#[test]
	fn combines_permissions() {
		let cases = vec![
			(vec!["/"], vec!["/"], 0),
			(vec!["/"], vec!["/a/b"], 4),
			(vec!["/a/b", "/a/f"], vec!["/a/c"], 2),
			(vec!["/a/c", "/a/b", "/a/f"], vec!["/a/c/d", "/a/b"], 2),
			(vec!["/a/c/d", "/g"], vec!["/a/c"], 1),
			(
				vec!["/a/c/d", "/a/c/e", "/a/f", "/g"],
				vec!["/a/f", "/a/c/e"],
				2,
			),
			(vec!["/a"], vec!["/g", "/a/b/c"], 4),
			(vec!["/a/b/c", "/a/c"], vec!["/a/c/e"], 1),
		]
		.into_iter()
		.map(|(only, except, n)| {
			(
				only.iter().map(PathBuf::from).collect(),
				except.iter().map(PathBuf::from).collect(),
				n,
			)
		});

		for (only, except, amount) in cases {
			let filter = Filter { only, except };

			let found = find_within_complex(&filter);
			assert_valid(found, amount);
		}

		let found = find_within_minimal(&Filter {
			except: vec![PathBuf::from("/")],
			only: vec![PathBuf::from("/")],
		});

		assert_valid(found, 0);
	}
}
