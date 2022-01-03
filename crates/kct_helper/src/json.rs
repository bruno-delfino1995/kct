use serde_json::Value;

// https://github.com/serde-rs/json/issues/377
pub fn merge(left: &mut Value, right: &Value) {
	match (left, right) {
		(&mut Value::Object(ref mut left), &Value::Object(ref right)) => {
			for (key, value) in right {
				merge(left.entry(key.clone()).or_insert(Value::Null), value);
			}
		}
		(a, b) => {
			*a = b.clone();
		}
	}
}

#[cfg(test)]
mod test {
	use super::*;

	fn json(contents: &str) -> Value {
		serde_json::from_str(contents).unwrap()
	}

	#[test]
	fn merge_when_keys_are_objects() {
		let mut left = json(r#"{"a": 1}"#);
		let right = json(r#"{"b": 1}"#);

		let expected = json(r#"{"a": 1, "b": 1}"#);

		merge(&mut left, &right);

		assert_eq!(left, expected)
	}

	#[test]
	fn replace_when_keys_match() {
		let mut left = json(r#"{"a": 1}"#);
		let right = json(r#"{"a": 2}"#);

		merge(&mut left, &right);

		assert_eq!(left, right)
	}

	#[test]
	fn goes_deep_for_objects() {
		let mut left = json(r#"{"a": {"b": {"c": 1}, "d": 1}, "e": 1}"#);
		let right = json(r#"{"a": {"b": {"d": 1}, "e": 1}, "f": 1}"#);

		let expected = json(r#"{"a": {"b": {"c": 1, "d": 1}, "d": 1, "e": 1}, "e": 1, "f": 1}"#);

		merge(&mut left, &right);

		assert_eq!(left, expected)
	}

	#[test]
	fn replaces_when_types_differ() {
		let mut left = json(r#"1"#);
		let right = json(r#""string""#);

		merge(&mut left, &right);

		assert_eq!(left, right)
	}
}
