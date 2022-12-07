use serde_json::{json, Map, Value};

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

pub fn get_in<'a>(obj: &'a Value, path: &[&str]) -> Option<&'a Value> {
	if !obj.is_object() {
		return None;
	}

	let mut path = path.iter().peekable();
	path.peek()?;

	let mut value = Some(obj);
	loop {
		let key = match path.next() {
			Some(k) => k,
			None => break value,
		};

		match value.and_then(|v| v.get(key)) {
			None => break None,
			Some(v @ Value::Object(_)) => value = Some(v),
			v @ Some(_) => {
				if path.peek().is_some() {
					break None;
				} else {
					value = v;
				}
			}
		}
	}
}

pub fn set_in<'a>(target: &'a mut Value, path: &[&str], value: Value) -> &'a Value {
	if target.is_null() {
		*target = Value::Object(Map::new());
	}

	if !target.is_object() {
		return target;
	}

	let changes = build_path(path, value);

	merge(target, &changes);

	target
}

fn build_path(path: &[&str], value: Value) -> Value {
	let mut path = path.iter();

	let mut base = json!({});
	let mut obj = &mut base;

	let mut a;
	let mut b = path.next();

	loop {
		a = b;
		b = path.next();

		match (a, &b) {
			(Some(key), Some(_)) => {
				obj = &mut obj[key];
			}

			(Some(s), None) => {
				let mut map = Map::new();
				map.insert(String::from(*s), value);

				*obj = Value::Object(map);
				break;
			}

			(None, _) => {
				break;
			}
		}
	}

	base
}

#[cfg(test)]
mod test {
	use serde_json::{json, Value};

	use super::{get_in, merge, set_in};

	mod get_path {
		use super::*;

		#[test]
		fn none_when_not_object() {
			let path = vec!["prop", "key"];

			assert_eq!(get_in(&Value::Null, &path), None);
			assert_eq!(get_in(&json!(1), &path), None);
			assert_eq!(get_in(&json!(1.2), &path), None);
			assert_eq!(get_in(&json!(true), &path), None);
			assert_eq!(get_in(&json!("value"), &path), None);
		}

		#[test]
		fn none_when_path_not_found() {
			assert_eq!(
				get_in(&json!({"a": {"b": {"c": 1}}}), &["c", "b", "a"]),
				None
			);
			assert_eq!(
				get_in(&json!({"a": {"b": {"c": 1}}}), &["z", "y", "x"]),
				None
			);
		}

		#[test]
		fn none_when_path_doesnt_end() {
			assert_eq!(
				get_in(&json!({"a": {"b": {"c": 1}}}), &["a", "b", "c", "d"]),
				None
			);
		}

		#[test]
		fn none_when_doesnt_find_object() {
			assert_eq!(get_in(&json!({"a": 1}), &["a", "b"]), None);
			assert_eq!(get_in(&json!({"a": {"b": 1}}), &["a", "b", "c"]), None);
		}

		#[test]
		fn finds_in_depth() {
			assert_eq!(
				get_in(&json!({"a": {"b": {"c": 1}}}), &["a", "b", "c"]),
				Some(&json!(1))
			);
		}
	}

	mod set_path {
		use super::*;

		#[test]
		fn changes_only_null_and_object() {
			let path = vec!["prop", "key"];
			let stay_put = vec![json!(1), json!(1.2), json!(true), json!("value")];

			for mut source in stay_put {
				let before = source.clone();

				let source = set_in(&mut source, &path, Value::Bool(true));

				assert_eq!(source, &before);
			}

			let expected = json!({"prop": {"key": true}});
			assert_eq!(
				set_in(&mut Value::Null, &path, Value::Bool(true)),
				&expected
			);
			assert_eq!(set_in(&mut json!({}), &path, Value::Bool(true)), &expected);
		}

		#[test]
		fn overwrites_in_depth() {
			let base = json!({"a": {"b": {"c": {"d": 1}}}});

			assert_eq!(
				set_in(&mut base.clone(), &["a"], Value::Bool(true)),
				&json!({"a": true})
			);
			assert_eq!(
				set_in(&mut base.clone(), &["a", "b"], Value::Bool(true)),
				&json!({"a": {"b": true}})
			);
			assert_eq!(
				set_in(&mut base.clone(), &["a", "b", "c"], Value::Bool(true)),
				&json!({"a": {"b": {"c": true}}})
			);
			assert_eq!(
				set_in(&mut base.clone(), &["a"], json!({"b": {"c": {"d": 1}}})),
				&base
			);
		}
	}

	mod merge {
		use super::*;

		#[test]
		fn merge_when_keys_are_objects() {
			let mut left = json!({"a": 1});
			let right = json!({"b": 1});

			let expected = json!({"a": 1, "b": 1});

			merge(&mut left, &right);

			assert_eq!(left, expected)
		}
	}
}
