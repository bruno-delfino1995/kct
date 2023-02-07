use std::iter;
use std::path::PathBuf;

use anyhow::Result;
use assert_matches::assert_matches;
use kct_kube::{error, Error, Kube, Manifest};
use kct_testing::compile;
use serde_json::{json, Value};

fn manifest() -> Value {
	json!({
			"kind": "Deployment",
			"apiVersion": "apps/v1"
	})
}

type Return = Result<Vec<Manifest>, Error>;

fn find_from(val: Value) -> Return {
	let kube = Kube::builder().value(val).build()?;

	Ok(kube.into())
}

fn assert_manifests(ok: Return, times: usize) {
	assert!(ok.is_ok());

	let manifest = manifest();
	let manifests: Vec<Value> = iter::repeat(manifest).take(times).collect();

	let rendered: Vec<Value> = ok
		.unwrap()
		.into_iter()
		.map(|manifest| {
			let (_, val) = manifest.into();

			val
		})
		.collect();
	assert_eq!(rendered, manifests)
}

fn assert_paths(ok: Return, paths: Vec<&str>) {
	assert!(ok.is_ok());

	let paths: Vec<PathBuf> = paths.into_iter().map(PathBuf::from).collect();
	let rendered: Vec<PathBuf> = ok
		.unwrap()
		.into_iter()
		.map(|manifest| {
			let (path, _): (PathBuf, Value) = manifest.into();

			path
		})
		.collect();
	assert_eq!(rendered, paths)
}

fn render(contents: &str) -> Value {
	let main = format!(
		r#"
		local _ = import 'kct.libsonnet';
		local sdk = _.sdk;
		local manifest(kind = 'Deployment') = {{
			kind: kind,
			apiVersion: 'apps/v1',
		}};

		{contents}
	"#
	);

	compile(main.as_str())
}

mod manifests {
	use super::*;

	#[test]
	fn finds_manifests() {
		let json = manifest();
		let found = find_from(json);
		assert_manifests(found, 1);

		let json = json!({"a": manifest(), "b": manifest()});
		let found = find_from(json);
		assert_manifests(found, 2);

		let json = json!({
			"a": {
			"b": manifest(),
			"c": {"d": manifest()},
			"e": manifest()
			}
		});
		let found = find_from(json);
		assert_manifests(found, 3);
	}

	#[test]
	fn disallow_primitives() {
		let values = [
			json!(0),
			json!("obj"),
			json!(null),
			json!({"a": 1, "b": manifest()}),
			json!({"a": {"b":{"c": null,"d": manifest()}}}),
			json!({"a": {"b": manifest()}, "c": "str"}),
		];

		for json in values.into_iter() {
			let error = find_from(json).unwrap_err();
			assert_matches!(error, Error::Output(error::Output::NotObject));
		}
	}
}

mod paths {
	use super::*;

	#[test]
	fn uses_prop_names_as_paths() {
		let json = json!({"a": {"b": manifest()}, "c": {"d": {"e":manifest()}}, "b": manifest()});
		let found = find_from(json);

		assert_paths(found, vec!["/a/b", "/b", "/c/d/e"])
	}

	#[test]
	fn allows_only_valid_and_clear_path_segments() {
		let json = json!({ "01-manifest": manifest() });
		let found = find_from(json);
		assert_paths(found, vec!["/01-manifest"]);

		let cases = vec![
			json!({ "/": manifest() }),
			json!({ "a/b": manifest() }),
			json!({ ".": manifest() }),
			json!({ "..": manifest() }),
			json!({ "some%thing": manifest() }),
			json!({ "this.complicates.filtering": manifest() }),
			json!({ "-start-alphanumeric": manifest() }),
			json!({ "end-alphanumeric-": manifest() }),
		];

		for j in cases.into_iter() {
			let error = find_from(j).unwrap_err();
			assert_matches!(error, Error::Output(error::Output::Path(_)))
		}
	}

	#[test]
	fn disallow_unclear_paths() {
		let cases = vec![
			json!([manifest(), manifest()]),
			json!({"a": manifest(), "b": [manifest(), manifest()]}),
			json!({"a": {"b": manifest(), "c": [manifest(), manifest()]}}),
		];

		for j in cases.into_iter() {
			let error = find_from(j).unwrap_err();
			assert_matches!(error, Error::Output(error::Output::NotObject));
		}
	}

	#[test]
	fn orders_props_alphabetically() {
		let json = json!({"z": manifest(), "1": manifest(), "01": manifest(), "10": manifest(), "2": manifest(), "a": manifest()});
		let found = find_from(json);

		assert_paths(found, vec!["/01", "/1", "/10", "/2", "/a", "/z"]);

		let json = json!({"a": {"c": manifest(), "b": manifest()}, "c": manifest(), "b": {"z": manifest(), "01": manifest(), "a": manifest(), "0": manifest()}});
		let found = find_from(json);

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
				a: sdk.inOrder(['b'], {b: sdk.inOrder(['c'], {c: sdk.inOrder(['d'], {d: sdk.inOrder(['e'], {e: sdk.inOrder(['g', 'f'], {f: manifest(), g: manifest()})})})})}),
				})"#
			),
			vec!["/a/b/c/d/e/g", "/a/b/c/d/e/f"]
		));

		cases.push((
			render(
				r#"sdk.inOrder(['b', 'a'], {
				a: manifest(),
				b: manifest()
				})"#,
			),
			vec!["/b", "/a"],
		));

		cases.push((
			render(
				r#"sdk.inOrder(['b'], {
				a: manifest(),
				b: manifest(),
				c: manifest()
				})"#,
			),
			vec!["/b", "/a", "/c"],
		));

		cases.push((
			render(
				r#"sdk.inOrder(['e', 'a'], {
				a: sdk.inOrder(['b','d','c'], {b: manifest(), c: manifest(), d: manifest()}),
				e: sdk.inOrder(['h', 'g', 'f'], {f: manifest(), g: manifest(), h: manifest()})
				})"#,
			),
			vec!["/e/h", "/e/g", "/e/f", "/a/b", "/a/d", "/a/c"],
		));

		cases.push((
			render(
				r#"{
				a: sdk.inOrder(['c', 'b'], {b: manifest(), c: manifest()}),
				d: sdk.inOrder(['e', 'f', 'i'], {
					e: manifest(),
					f: sdk.inOrder(['h', 'g'], {g: manifest(), h: manifest()}),
					i: manifest()
				})
				}"#,
			),
			vec!["/a/c", "/a/b", "/d/e", "/d/f/h", "/d/f/g", "/d/i"],
		));

		cases.push((
			render(
				r#"sdk.inOrder(['b', 'a'], {
				a: sdk.inOrder(['b'], {b: {c: {b: sdk.inOrder(['d'], {d: {a: manifest()}, c: manifest()})}}, a: manifest()}),
				b: { d: {d: sdk.inOrder(['d'], {c: sdk.inOrder(['c'], {c: {a: manifest()}, a: manifest()}), d: manifest()})}}
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
					a: manifest(),
					b: sdk.inOrder(['k','c'], {
						c: manifest(),
						d: sdk.inOrder(['e', 'h', 'g'], {
							e: {f: manifest()},
							g: manifest(),
							h: sdk.inOrder(['j', 'i'], {i: manifest(), j: manifest()})
						}),
						k: manifest(),
						l: {m: manifest()}
					})
				}),
				y: manifest()
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

		for (json, order) in cases.into_iter() {
			let found = find_from(json);
			assert_paths(found, order);
		}
	}

	#[test]
	fn fails_on_invalid_annotation() {
		let json = render(
			r#"{a: manifest() + { metadata+: { annotations+: { 'kct.io/order': 'a:1' }} }}"#,
		);
		let error = find_from(json).unwrap_err();

		assert_matches!(
			error,
			Error::Object(error::Object::Tracking(error::Tracking::Format))
		);

		let json = render(
			r#"{a: manifest() + { metadata+: { annotations+: { 'kct.io/order': '-:1:1' }} }}"#,
		);
		let error = find_from(json).unwrap_err();

		assert_matches!(error, Error::Object(error::Object::Tracking(error::Tracking::InvalidPart(field))) => {
			assert_eq!(field, "field")
		});

		let json = render(
			r#"{a: manifest() + { metadata+: { annotations+: { 'kct.io/order': 'a:a:1' }} }}"#,
		);
		let error = find_from(json).unwrap_err();

		assert_matches!(error, Error::Object(error::Object::Tracking(error::Tracking::InvalidPart(field))) => {
			assert_eq!(field, "depth")
		});

		let json = render(
			r#"{a: manifest() + { metadata+: { annotations+: { 'kct.io/order': 'a:1:b' }} }}"#,
		);
		let error = find_from(json).unwrap_err();

		assert_matches!(error, Error::Object(error::Object::Tracking(error::Tracking::InvalidPart(field))) => {
			assert_eq!(field, "order")
		});
	}

	#[test]
	fn orders_props_by_kind() {
		let mut cases = vec![];

		cases.push((
			render(
				r#"{a: manifest(), b: manifest('Namespace'), c: manifest('Pod'), d: manifest('Job')}"#,
			),
			vec!["/b", "/c", "/a", "/d"],
		));

		cases.push((
			render(
				r#"sdk.inOrder(['c'], {
				a: manifest('Deployment'),
				b: manifest('Pod'),
				c: manifest('Job'),
									d: manifest('Namespace')
				})"#,
			),
			vec!["/c", "/d", "/b", "/a"],
		));

		cases.push((
			render(r#"{a: manifest(), b: manifest(), c: manifest('Secret')}"#),
			vec!["/c", "/a", "/b"],
		));

		cases.push((
			render(r#"{a: manifest(), b: manifest('Unknown'), c: manifest()}"#),
			vec!["/a", "/c", "/b"],
		));

		cases.push((
			render(
				r#"{
			x: {a: manifest(), b: manifest('Namespace')},
			y: {a: manifest('Secret'), b: manifest('Job')},
				}"#,
			),
			vec!["/x/b", "/x/a", "/y/a", "/y/b"],
		));

		cases.push((
			render(
				r#"sdk.inOrder(['y'], {
			x: manifest('Unknown'),
			y: {a: manifest('Secret'), b: manifest('Job')},
			z: {a: manifest(), b: manifest('Namespace')},
		})"#,
			),
			vec!["/y/a", "/y/b", "/x", "/z/b", "/z/a"],
		));

		cases.push((
			render(
				r#"sdk.inOrder(['d'], {
			a: manifest('Service'),
			c: {b: manifest('Pod'), c: manifest('Deployment')},
			b: manifest('Job'),
			d: manifest()
		})"#,
			),
			vec!["/d", "/a", "/b", "/c/b", "/c/c"],
		));

		for (json, order) in cases {
			let found = find_from(json);
			assert_paths(found, order);
		}
	}
}

mod filter {
	use super::*;

	fn find_within_minimal(only: Vec<PathBuf>, except: Vec<PathBuf>) -> Return {
		let kube = Kube::builder()
			.value(manifest())
			.only(only)
			.except(except)
			.build()?;

		Ok(kube.into())
	}

	fn find_within_complex(only: Vec<PathBuf>, except: Vec<PathBuf>) -> Return {
		let complex = json!({"a": {"b": manifest(), "c": {"d": manifest(), "e": manifest()}, "f": manifest()}, "g": manifest()});

		let kube = Kube::builder()
			.value(complex)
			.only(only)
			.except(except)
			.build()?;

		Ok(kube.into())
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
			let found = find_within_complex(only, vec![]);
			assert_manifests(found, amount);
		}

		let found = find_within_minimal(vec![PathBuf::from("/")], vec![]);

		assert_manifests(found, 1);
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
			let found = find_within_complex(vec![], except);
			assert_manifests(found, amount);
		}

		let found = find_within_minimal(vec![], vec![PathBuf::from("/")]);
		assert_manifests(found, 0);
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
			let found = find_within_complex(only, except);
			assert_manifests(found, amount);
		}

		let found = find_within_minimal(vec![PathBuf::from("/")], vec![PathBuf::from("/")]);
		assert_manifests(found, 0);
	}
}
