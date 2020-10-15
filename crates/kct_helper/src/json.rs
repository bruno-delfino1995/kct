use serde_json::Value;

// https://github.com/serde-rs/json/issues/377
pub fn merge(a: &mut Value, b: &Value) {
	match (a, b) {
		(&mut Value::Object(ref mut a), &Value::Object(ref b)) => {
			for (k, v) in b {
				merge(a.entry(k.clone()).or_insert(Value::Null), v);
			}
		}
		(a, b) => {
			*a = b.clone();
		}
	}
}
