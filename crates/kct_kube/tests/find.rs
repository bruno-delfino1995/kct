use kct_kube::{Error, Filter, Result};
use kct_testing::compile;
use serde_json::json;
use serde_json::Value;
use std::iter;
use std::path::PathBuf;

fn obj() -> Value {
	json!({
		"kind": "Deployment",
		"apiVersion": "apps/v1"
	})
}

type Return = Result<Vec<(PathBuf, Value)>>;

fn find_from(val: &Value) -> Return {
	kct_kube::find(val, &Filter::default())
}

fn assert_invalid(err: Return) {
	assert!(err.is_err());
	assert_eq!(err.unwrap_err(), Error::Invalid);
}

fn assert_objects(ok: Return, times: usize) {
	assert!(ok.is_ok());

	let obj = obj();
	let objs: Vec<Value> = iter::repeat(obj).take(times).collect();

	let rendered: Vec<Value> = ok
		.unwrap()
		.into_iter()
		.map(|(_path, value)| value)
		.collect();
	assert_eq!(rendered, objs)
}

fn assert_paths(ok: Return, paths: Vec<&str>) {
	assert!(ok.is_ok());

	let paths: Vec<PathBuf> = paths.into_iter().map(PathBuf::from).collect();
	let rendered: Vec<PathBuf> = ok.unwrap().into_iter().map(|(path, _value)| path).collect();
	assert_eq!(rendered, paths)
}

fn render(contents: &str) -> Value {
	let main = format!(
		r#"
		local _ = import 'kct.libsonnet';
		local sdk = _.sdk;
        local obj(kind = 'Deployment') = {{
			kind: kind,
			apiVersion: 'apps/v1',
		}};

		{contents}
	"#
	);

	compile(main.as_str())
}

mod objects {
	use super::*;

	#[test]
	fn finds_objects() {
		let json = obj();
		let found = find_from(&json);
		assert_objects(found, 1);

		let json = json!({"a": obj(), "b": obj()});
		let found = find_from(&json);
		assert_objects(found, 2);

		let json = json!({
			"a": {
				"b": obj(),
				"c": {"d": obj()},
				"e": obj()
			}
		});
		let found = find_from(&json);
		assert_objects(found, 3);
	}

	#[test]
	fn disallow_primitives() {
		let values = [
			json!(0),
			json!("obj"),
			json!(null),
			json!({"a": 1, "b": obj()}),
			json!({"a": {"b":{"c": null,"d": obj()}}}),
			json!({"a": {"b": obj()}, "c": "str"}),
		];

		for json in values.iter() {
			let found = find_from(json);
			assert_invalid(found);
		}
	}
}

mod paths {
	use super::*;

	#[test]
	fn uses_prop_names_as_paths() {
		let json = json!({"a": {"b": obj()}, "c": {"d": {"e":obj()}}, "b": obj()});
		let found = find_from(&json);

		assert_paths(found, vec!["/a/b", "/b", "/c/d/e"])
	}

	#[test]
	fn orders_props_alphabetically() {
		let json =
			json!({"z": obj(), "1": obj(), "01": obj(), "10": obj(), "2": obj(), "a": obj()});
		let found = find_from(&json);

		assert_paths(found, vec!["/01", "/1", "/10", "/2", "/a", "/z"]);

		let json = json!({"a": {"c": obj(), "b": obj()}, "c": obj(), "b": {"z": obj(), "01": obj(), "a": obj(), "0": obj()}});
		let found = find_from(&json);

		assert_paths(
			found,
			vec!["/a/b", "/a/c", "/b/0", "/b/01", "/b/a", "/b/z", "/c"],
		)
	}

	#[test]
	fn orders_props_by_annotation() {
		let mut cases = vec![];

		cases.push((
			render(
				r#"sdk.inOrder(['a'], {
					a: sdk.inOrder(['b'], {b: sdk.inOrder(['c'], {c: sdk.inOrder(['d'], {d: sdk.inOrder(['e'], {e: sdk.inOrder(['g', 'f'], {f: obj(), g: obj()})})})})}),
				})"#
			),
			vec!["/a/b/c/d/e/g", "/a/b/c/d/e/f"]
		));

		cases.push((
			render(
				r#"sdk.inOrder(['b', 'a'], {
					a: obj(),
					b: obj()
				})"#,
			),
			vec!["/b", "/a"],
		));

		cases.push((
			render(
				r#"sdk.inOrder(['b'], {
					a: obj(),
					b: obj(),
					c: obj()
				})"#,
			),
			vec!["/b", "/a", "/c"],
		));

		cases.push((
			render(
				r#"sdk.inOrder(['e', 'a'], {
					a: sdk.inOrder(['b','d','c'], {b: obj(), c: obj(), d: obj()}),
					e: sdk.inOrder(['h', 'g', 'f'], {f: obj(), g: obj(), h: obj()})
				})"#,
			),
			vec!["/e/h", "/e/g", "/e/f", "/a/b", "/a/d", "/a/c"],
		));

		cases.push((
			render(
				r#"{
					a: sdk.inOrder(['c', 'b'], {b: obj(), c: obj()}),
					d: sdk.inOrder(['e', 'f', 'i'], {
						e: obj(),
						f: sdk.inOrder(['h', 'g'], {g: obj(), h: obj()}),
						i: obj()
					})
				}"#,
			),
			vec!["/a/c", "/a/b", "/d/e", "/d/f/h", "/d/f/g", "/d/i"],
		));

		cases.push((
			render(
				r#"sdk.inOrder(['b', 'a'], {
					a: sdk.inOrder(['b'], {b: {c: {b: sdk.inOrder(['d'], {d: {a: obj()}, c: obj()})}}, a: obj()}),
					b: { d: {d: sdk.inOrder(['d'], {c: sdk.inOrder(['c'], {c: {a: obj()}, a: obj()}), d: obj()})}}
				})"#,
			),
			vec![
				"/b/d/d/d",
				"/b/d/d/c/c/a",
				"/b/d/d/c/a",
				"/a/b/c/b/d/a",
				"/a/b/c/b/c",
				"/a/a",
			],
		));

		cases.push((
			render(
				r#"sdk.inOrder(['y', 'x'], {
					x: sdk.inOrder(['b', 'a'], {
						a: obj(),
						b: sdk.inOrder(['k','c'], {
							c: obj(),
							d: sdk.inOrder(['e', 'h', 'g'], {
								e: {f: obj()},
								g: obj(),
								h: sdk.inOrder(['j', 'i'], {i: obj(), j: obj()})
							}),
							k: obj(),
							l: {m: obj()}
						})
					}),
					y: obj()
				})"#,
			),
			vec![
				"/y",
				"/x/b/k",
				"/x/b/c",
				"/x/b/d/e/f",
				"/x/b/d/h/j",
				"/x/b/d/h/i",
				"/x/b/d/g",
				"/x/b/l/m",
				"/x/a",
			],
		));

		for (json, order) in cases {
			let found = find_from(&json);
			assert_paths(found, order);
		}
	}

	#[test]
	fn orders_props_by_kind() {
		let mut cases = vec![];

		cases.push((
			render(r#"{a: obj(), b: obj('Namespace'), c: obj('Pod'), d: obj('Job')}"#),
			vec!["/b", "/c", "/a", "/d"],
		));

		cases.push((
			render(
				r#"sdk.inOrder(['c'], {
					a: obj('Deployment'),
					b: obj('Pod'),
					c: obj('Job'),
                    d: obj('Namespace')
				})"#,
			),
			vec!["/c", "/d", "/b", "/a"],
		));

		cases.push((
			render(r#"{a: obj(), b: obj(), c: obj('Secret')}"#),
			vec!["/c", "/a", "/b"],
		));

		cases.push((
			render(r#"{a: obj(), b: obj('Unknown'), c: obj()}"#),
			vec!["/a", "/c", "/b"],
		));

		cases.push((
			render(
				r#"{
				x: {a: obj(), b: obj('Namespace')},
				y: {a: obj('Secret'), b: obj('Job')},
			}"#,
			),
			vec!["/x/b", "/x/a", "/y/a", "/y/b"],
		));

		cases.push((
			render(
				r#"sdk.inOrder(['y'], {
				x: obj('Unknown'),
				y: {a: obj('Secret'), b: obj('Job')},
				z: {a: obj(), b: obj('Namespace')},
			})"#,
			),
			vec!["/y/a", "/y/b", "/x", "/z/b", "/z/a"],
		));

		cases.push((
			render(
				r#"sdk.inOrder(['d'], {
				a: obj('Service'),
				c: {b: obj('Pod'), c: obj('Deployment')},
				b: obj('Job'),
				d: obj()
			})"#,
			),
			vec!["/d", "/a", "/b", "/c/b", "/c/c"],
		));

		for (json, order) in cases {
			let found = find_from(&json);
			assert_paths(found, order);
		}
	}

	#[test]
	fn allows_only_valid_and_clear_path_segments() {
		let json = json!({ "01-obj": obj() });
		let found = find_from(&json);
		assert_paths(found, vec!["/01-obj"]);

		let cases = vec![
			json!({ "/": obj() }),
			json!({ "a/b": obj() }),
			json!({ ".": obj() }),
			json!({ "..": obj() }),
			json!({ "some%thing": obj() }),
			json!({ "this.complicates.filtering": obj() }),
			json!({ "-start-alphanumeric": obj() }),
			json!({ "end-alphanumeric-": obj() }),
		];

		for j in cases {
			let found = find_from(&j);
			assert_invalid(found)
		}
	}

	#[test]
	fn disallow_unclear_paths() {
		let cases = vec![
			json!([obj(), obj()]),
			json!({"a": obj(), "b": [obj(), obj()]}),
			json!({"a": {"b": obj(), "c": [obj(), obj()]}}),
		];

		for j in cases {
			let found = find_from(&j);
			assert_invalid(found)
		}
	}
}

mod filter {
	use super::*;

	fn find_within_minimal(filter: &Filter) -> Return {
		kct_kube::find(&obj(), filter)
	}

	fn find_within_complex(filter: &Filter) -> Return {
		let complex =
			json!({"a": {"b": obj(), "c": {"d": obj(), "e": obj()}, "f": obj()}, "g": obj()});

		kct_kube::find(&complex, filter)
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
			assert_objects(found, amount);
		}

		let found = find_within_minimal(&Filter {
			except: vec![],
			only: vec![PathBuf::from("/")],
		});

		assert_objects(found, 1);
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
			assert_objects(found, amount);
		}

		let found = find_within_minimal(&Filter {
			except: vec![PathBuf::from("/")],
			only: vec![],
		});

		assert_objects(found, 0);
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
			assert_objects(found, amount);
		}

		let found = find_within_minimal(&Filter {
			except: vec![PathBuf::from("/")],
			only: vec![PathBuf::from("/")],
		});

		assert_objects(found, 0);
	}
}
