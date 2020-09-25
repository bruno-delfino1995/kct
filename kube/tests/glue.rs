use serde_json::Value;
use std::iter;

const MINIMAL_OBJECT: &str = r#"{ "kind": "Deployment", "apiVersion": "apps/v1" }"#;

fn generate(times: usize) -> Vec<Value> {
	let object = serde_json::from_str(MINIMAL_OBJECT).unwrap();
	iter::repeat(object).take(times).collect()
}

#[test]
fn returns_plain_on_singleton_lists() {
	let objects = generate(1);

	let glued = kube::glue(&objects);

	let object: Value = serde_json::from_str(MINIMAL_OBJECT).unwrap();
	assert_eq!(glued, object);
}

#[test]
fn returns_list_on_list() {
	let amount = 3;
	let objects = generate(amount);

	let glued = kube::glue(&objects);

	let json = format!(
		r#"{{ "kind": "List", "apiVersion": "v1", "items": [{0}, {0}, {0}] }}"#,
		MINIMAL_OBJECT
	);
	let object: Value = serde_json::from_str(&json).unwrap();

	assert_eq!(glued, object);
}
