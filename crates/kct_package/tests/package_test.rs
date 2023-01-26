use kct_testing::{self as testing, dir::TempDir, Fixture};

use kct_package::{Error, Package, Release};
use serde_json::{json, Map, Value};
use std::convert::TryFrom;
use std::panic::panic_any;
use std::path::PathBuf;

fn package(with: Vec<(&str, &str)>, without: Vec<&str>) -> (Result<Package, Error>, TempDir) {
	let dir = Fixture::custom(with, without);
	let package = Package::try_from(dir.path());

	(package, dir)
}

fn compile_with_example(pkg: Package, rel: Option<Release>) -> Result<Value, Error> {
	let input = pkg.example.clone().unwrap();

	pkg.compile(Some(input), rel)
}

mod try_from {
	use super::*;

	#[test]
	fn can_be_created() {
		let (package, _dir) = package(vec![], vec![]);

		assert!(package.is_ok());
	}

	#[test]
	fn need_spec() {
		let (package, _dir) = package(vec![], vec!["kcp.json"]);

		assert!(package.is_err());
		assert_eq!(package.unwrap_err(), Error::NoSpec)
	}

	#[test]
	fn requests_example_for_schema() {
		let (package, _dir) = package(vec![], vec!["example.json"]);

		assert!(package.is_err());
		assert_eq!(package.unwrap_err(), Error::NoExample)
	}

	#[test]
	fn request_schema_for_input() {
		let (package, _dir) = package(vec![], vec!["schema.json"]);

		assert!(package.is_err());
		assert_eq!(package.unwrap_err(), Error::NoSchema)
	}

	#[test]
	fn example_should_be_valid_input() {
		let (package, _dir) = package(vec![("example.json", r#"{"nothing": "none"}"#)], vec![]);

		assert!(package.is_err());
		assert_eq!(package.unwrap_err(), Error::InvalidExample);
	}

	#[test]
	fn needs_a_main_file() {
		let (package, _dir) = package(vec![], vec!["templates/main.jsonnet"]);

		assert!(package.is_err());
		assert_eq!(package.unwrap_err(), Error::NoMain);
	}
}

mod compile {
	use super::*;

	mod input {
		use super::*;

		#[test]
		fn renders_with_null() {
			let (package, _dir) = package(
				vec![("templates/main.jsonnet", "(import 'kct.libsonnet').input")],
				vec!["example.json", "schema.json"],
			);
			let package = package.unwrap();

			let rendered = package.compile(None, None);

			assert_eq!(rendered.unwrap(), Value::Null);
		}

		#[test]
		fn doesnt_merge_input_with_defaults() {
			let input: Value = testing::json(
				r#"{ "database": { "port": 5432, "host": "localhost", "credentials": { "user": "admin", "pass": "admin" } } }"#,
			);

			let (package, _dir) = package(
				vec![("templates/main.jsonnet", "(import 'kct.libsonnet').input")],
				vec![],
			);
			let package = package.unwrap();

			let rendered = package.compile(Some(input.clone()), None);

			assert_eq!(rendered.unwrap(), input);
		}

		#[test]
		#[should_panic(expected = "input provided doesn't match the schema")]
		fn validate_input() {
			let input: Value = testing::json(r#"{ "database": null }"#);

			let (root, _dir) = package(
				vec![("templates/main.jsonnet", "(import 'kct.libsonnet').input")],
				vec![],
			);
			let package = root.unwrap();

			let rendered = package.compile(Some(input), None).unwrap_err();

			match rendered {
				Error::InvalidInput => panic_any(rendered.to_string()),
				_ => panic!("It should be a validation issue!"),
			}
		}
	}

	mod jsonnet {
		use super::*;

		#[test]
		#[should_panic(expected = "manifest function")]
		fn disallows_top_level_functions() {
			let (package, _dir) = package(
				vec![(
					"templates/main.jsonnet",
					"function(input = null, files = null) { input: input }",
				)],
				vec![],
			);

			let package = package.unwrap();
			let rendered = compile_with_example(package, None).unwrap_err();

			match rendered {
				Error::RenderIssue(err) => panic_any(err),
				_ => panic!("It should be a render issue!"),
			}
		}

		#[test]
		fn renders_imports() {
			let (package, _dir) = package(
				vec![
					(
						"templates/main.jsonnet",
						"local valid = import './input/entry.jsonnet'; valid",
					),
					("templates/input/entry.jsonnet", "import '../input.jsonnet'"),
					("templates/input.jsonnet", "(import 'kct.libsonnet').input"),
				],
				vec![],
			);

			let package = package.unwrap();
			let input = package.example.clone().unwrap();
			let rendered = compile_with_example(package, None);

			assert_eq!(rendered.unwrap(), input);
		}

		#[test]
		#[should_panic(expected = "can't resolve input.jsonnet")]
		fn doesnt_include_templates_on_imports() {
			let (package, _dir) = package(
				vec![
					(
						"templates/main.jsonnet",
						"local valid = import './input/entry.jsonnet'; valid",
					),
					("templates/input/entry.jsonnet", "import 'input.jsonnet'"),
					("templates/input.jsonnet", "(import 'kct.libsonnet').input"),
				],
				vec![],
			);

			let package = package.unwrap();
			let rendered = compile_with_example(package, None).unwrap_err();

			match rendered {
				Error::RenderIssue(err) => panic_any(err),
				_ => panic!("It should be a render issue!"),
			}
		}

		#[test]
		fn includes_vendor_for_imports() {
			let (package, _dir) = package(
				vec![
					(
						"templates/main.jsonnet",
						"local valid = import 'ksonnet/ksonnet.beta.4/k8s.libjsonnet'; valid",
					),
					(
						"vendor/ksonnet/ksonnet.beta.4/k8s.libjsonnet",
						"(import 'kct.libsonnet').input",
					),
				],
				vec![],
			);

			let package = package.unwrap();
			let input = package.example.clone().unwrap();
			let rendered = compile_with_example(package, None);

			assert_eq!(rendered.unwrap(), input);
		}

		#[test]
		fn includes_lib_for_aliasing() {
			let (package, _dir) = package(
				vec![
					(
						"templates/main.jsonnet",
						"local valid = import 'k.libjsonnet'; valid",
					),
					(
						"vendor/ksonnet/ksonnet.beta.4/k8s.libjsonnet",
						"(import 'kct.libsonnet').input",
					),
					(
						"lib/k.libjsonnet",
						"import 'ksonnet/ksonnet.beta.4/k8s.libjsonnet'",
					),
				],
				vec![],
			);

			let package = package.unwrap();
			let input = package.example.clone().unwrap();
			let rendered = compile_with_example(package, None);

			assert_eq!(rendered.unwrap(), input);
		}
	}

	mod file_templates {
		use super::*;

		#[test]
		fn renders_templates() {
			let (package, _dir) = package(
				vec![(
					"templates/main.jsonnet",
					"(import 'kct.libsonnet').files('database.toml')",
				)],
				vec![],
			);
			let package = package.unwrap();
			let input = package.example.clone().unwrap();
			let template = testing::template(&Fixture::contents("files/database.toml"), &input);
			let rendered = package.compile(Some(input), None);

			assert_eq!(rendered.unwrap(), Value::String(template));
		}

		#[test]
		fn renders_multiple_templates() {
			let (package, _dir) = package(
				vec![(
					"templates/main.jsonnet",
					"(import 'kct.libsonnet').files('**/*.toml')",
				)],
				vec![],
			);
			let package = package.unwrap();
			let input = package.example.clone().unwrap();

			let db_template = testing::template(&Fixture::contents("files/database.toml"), &input);
			let evt_template =
				testing::template(&Fixture::contents("files/events/settings.toml"), &input);

			let rendered = compile_with_example(package, None);

			assert_eq!(
				rendered.unwrap(),
				Value::Array(vec![
					Value::String(db_template),
					Value::String(evt_template),
				])
			);
		}

		#[test]
		#[should_panic(expected = "Unable to compile templates")]
		fn fails_on_invalid_templates() {
			let (package, _dir) = package(
				vec![(
					"templates/main.jsonnet",
					"(import 'kct.libsonnet').files('invalid.ini')",
				)],
				vec![],
			);
			let package = package.unwrap();

			let rendered = compile_with_example(package, None).unwrap_err();

			match rendered {
				Error::RenderIssue(err) => panic_any(err),
				_ => panic!("It should be a render issue!"),
			}
		}

		#[test]
		fn compiles_templates_with_empty_input() {
			let (package, _dir) = package(
				vec![(
					"templates/main.jsonnet",
					"(import 'kct.libsonnet').files('no-params.txt')",
				)],
				vec!["example.json", "schema.json"],
			);
			let package = package.unwrap();

			let template = testing::template(
				&Fixture::contents("files/no-params.txt"),
				&Value::Object(Map::new()),
			);

			let rendered = package.compile(None, None);

			assert_eq!(rendered.unwrap(), Value::String(template));
		}

		#[test]
		#[should_panic(expected = "No files folder to search for templates")]
		fn fails_on_empty_templates_folder() {
			let (package, _dir) = package(vec![], vec!["files"]);
			let package = package.unwrap();

			let rendered = compile_with_example(package, None).unwrap_err();

			match rendered {
				Error::RenderIssue(err) => panic_any(err),
				_ => panic!("It should be a render issue!"),
			}
		}

		#[test]
		#[should_panic(expected = "No template found for glob")]
		fn fails_on_not_found_template() {
			let (package, _dir) = package(
				vec![(
					"templates/main.jsonnet",
					"(import 'kct.libsonnet').files('*.json')",
				)],
				vec![],
			);
			let package = package.unwrap();

			let rendered = compile_with_example(package, None).unwrap_err();

			match rendered {
				Error::RenderIssue(err) => panic_any(err),
				_ => panic!("It should be a render issue!"),
			}
		}
	}

	mod release {
		use super::*;

		#[test]
		fn prefixes_installation_name() {
			let release = Release {
				name: String::from("rc"),
			};
			let (package, _dir) = package(
				vec![("templates/main.jsonnet", "(import 'kct.libsonnet').name")],
				vec![],
			);
			let package = package.unwrap();

			let json = json!(format!("{}-{}", release.name, package.spec.name));
			let rendered = compile_with_example(package, Some(release));

			assert_eq!(rendered.unwrap(), json);
		}

		#[test]
		fn is_injected_on_global() {
			let release = Release {
				name: String::from("rc"),
			};
			let (package, _dir) = package(
				vec![("templates/main.jsonnet", "(import 'kct.libsonnet').release")],
				vec![],
			);
			let package = package.unwrap();

			let json = json!({ "name": release.name });
			let rendered = compile_with_example(package, Some(release));

			assert_eq!(rendered.unwrap(), json);
		}
	}

	mod package {
		use super::*;

		#[test]
		fn is_injected_on_global() {
			let (package, _dir) = package(
				vec![("templates/main.jsonnet", "(import 'kct.libsonnet').package")],
				vec![],
			);
			let package = package.unwrap();

			let json = json!({
				"name": package.spec.name,
				"version": package.spec.version.to_string()
			});
			let rendered = compile_with_example(package, None);

			assert_eq!(rendered.unwrap(), json);
		}

		#[test]
		fn is_default_installation_name() {
			let (package, _dir) = package(
				vec![("templates/main.jsonnet", "(import 'kct.libsonnet').name")],
				vec![],
			);
			let package = package.unwrap();

			let json = json!(package.spec.name);
			let rendered = compile_with_example(package, None);

			assert_eq!(rendered.unwrap(), json);
		}
	}

	mod subpackage {
		use super::*;

		fn subpackage(dir: &TempDir, name: &str, with: Vec<(&str, &str)>, without: Vec<&str>) {
			let (_package, source) = package(with, without);

			let path = dir.path().join("vendor").join(name);
			testing::dir::mk(&path);
			testing::dir::mv(&source.into_path(), &path)
		}

		#[test]
		fn are_rendered_with_include() {
			let (root, dir) = package(
				vec![(
					"templates/main.jsonnet",
					"(import 'kct.libsonnet').include('sub', (import 'kct.libsonnet').input)",
				)],
				vec![],
			);
			subpackage(
				&dir,
				"sub",
				vec![("templates/main.jsonnet", "(import 'kct.libsonnet').input")],
				vec![],
			);
			let package = root.unwrap();
			let rendered = compile_with_example(package, None);

			let result = testing::json(&Fixture::contents("example.json"));

			assert_eq!(rendered.unwrap(), result);
		}

		// NOTE: As subpackages are supposed to be normal packages, we don't
		// need to validate all the same cases as we do for packages. This test
		// is usefull to guarantee that we pass the correct input while
		// compiling the subpackage, not to check the realm of invalid packages
		#[test]
		#[should_panic(expected = "input provided doesn't match the schema")]
		fn validate_input() {
			let (root, dir) = package(
				vec![(
					"templates/main.jsonnet",
					"(import 'kct.libsonnet').include('sub', { database: null })",
				)],
				vec![],
			);
			subpackage(
				&dir,
				"sub",
				vec![("templates/main.jsonnet", "(import 'kct.libsonnet').input")],
				vec![],
			);
			let package = root.unwrap();

			let rendered = compile_with_example(package, None).unwrap_err();

			match rendered {
				Error::RenderIssue(err) => panic_any(err),
				_ => panic!("It should be a render issue!"),
			}
		}

		#[test]
		fn has_same_release() {
			let name = "rc";
			let release = Release {
				name: String::from(name),
			};
			let (root, dir) = package(
				vec![(
					"templates/main.jsonnet",
					"(import 'kct.libsonnet').include('sub', (import 'kct.libsonnet').input)",
				)],
				vec![],
			);
			subpackage(
				&dir,
				"sub",
				vec![("templates/main.jsonnet", "(import 'kct.libsonnet').release")],
				vec![],
			);
			let package = root.unwrap();

			let rendered = compile_with_example(package, Some(release)).unwrap();

			let actual = rendered.get("name").unwrap().as_str().unwrap();
			assert_eq!(actual, name);
		}

		#[test]
		fn can_render_own_subpackages() {
			let contents = r#"{"omae_wha": "mou shindeiru"}"#;
			let (root, dir) = package(
				vec![(
					"templates/main.jsonnet",
					"(import 'kct.libsonnet').include('dep', (import 'kct.libsonnet').input)",
				)],
				vec![],
			);
			subpackage(
				&dir,
				"dep",
				vec![(
					"templates/main.jsonnet",
					"(import 'kct.libsonnet').include('transient', (import 'kct.libsonnet').input)",
				)],
				vec![],
			);
			subpackage(
				&dir,
				"transient",
				vec![("templates/main.jsonnet", contents)],
				vec![],
			);
			let package = root.unwrap();
			let rendered = compile_with_example(package, None);

			let result = testing::json(contents);

			assert_eq!(rendered.unwrap(), result);
		}
	}
}
