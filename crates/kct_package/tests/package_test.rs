mod fixtures;
mod helpers;

use fixtures::Fixture;
use kct_package::{error::Error, Package, Release};
use serde_json::{Map, Value};
use std::convert::TryFrom;
use std::fs;
use std::panic::panic_any;
use std::path::PathBuf;
use tempfile::TempDir;

fn package(with: Vec<(&str, &str)>, without: Vec<&str>) -> (Result<Package, Error>, TempDir) {
	use fs_extra::dir::{self as fsdir, CopyOptions};

	let dir = {
		let tempdir = TempDir::new().unwrap();
		let source = Fixture::path("kcp");

		let mut options = CopyOptions::new();
		options.content_only = true;
		fsdir::copy(source, tempdir.path(), &options).unwrap();

		tempdir
	};

	for (path, contents) in with {
		let to_add = dir.path().join(path);
		let parent = to_add.parent().unwrap();

		if !parent.exists() {
			fsdir::create_all(parent, false).unwrap();
		}

		fs::write(to_add, contents).unwrap();
	}

	for path in without {
		let to_remove = dir.path().join(path);

		if to_remove.is_dir() {
			fs::remove_dir_all(to_remove).unwrap();
		} else {
			fs::remove_file(to_remove).unwrap();
		}
	}

	let package = Package::try_from(PathBuf::from(dir.path()));

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
	fn from_archive() {
		let cwd = TempDir::new().unwrap();
		let (package, _dir) = package(vec![], vec![]);
		let package = package.unwrap();
		let archive = package.archive(&PathBuf::from(cwd.path())).unwrap();

		let package = Package::try_from(archive);

		assert!(package.is_ok())
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

mod archive {
	use super::*;

	#[test]
	fn creates_a_file_on_provided_dir() {
		let cwd = TempDir::new().unwrap();
		let (package, _dir) = package(vec![], vec![]);
		let package = package.unwrap();

		let compressed = package.archive(&PathBuf::from(cwd.path()));

		assert!(compressed.is_ok());
		assert!(compressed.unwrap().starts_with(cwd.path()));
	}

	#[test]
	fn creates_archive_with_spec_info() {
		let cwd = TempDir::new().unwrap();
		let (package, _dir) = package(vec![], vec![]);
		let package = package.unwrap();
		let name = format!("{}_{}", package.spec.name, package.spec.version);

		let compressed = package.archive(&PathBuf::from(cwd.path()));

		assert!(compressed.is_ok());
		assert_eq!(
			name,
			compressed.unwrap().file_stem().unwrap().to_str().unwrap()
		);
	}

	#[test]
	fn can_be_compiled_after_archived() {
		let cwd = TempDir::new().unwrap();
		let (package, _dir) = package(vec![], vec![]);
		let package = package.unwrap();

		let compressed = package.archive(&PathBuf::from(cwd.path())).unwrap();
		let package = Package::try_from(compressed).unwrap();
		let compiled = compile_with_example(package, None);

		assert!(compiled.is_ok());
	}
}

mod compile {
	use super::*;

	mod input {
		use super::*;

		#[test]
		fn renders_with_null() {
			let (package, _dir) = package(
				vec![("templates/main.jsonnet", "_.input")],
				vec!["example.json", "schema.json"],
			);
			let package = package.unwrap();

			let rendered = package.compile(None, None);

			assert_eq!(rendered.unwrap(), Value::Null);
		}

		#[test]
		fn doesnt_merge_input_with_defaults() {
			let input: Value = helpers::json(
				r#"{ "database": { "port": 5432, "host": "localhost", "credentials": { "user": "admin", "pass": "admin" } } }"#,
			);

			let (package, _dir) = package(vec![("templates/main.jsonnet", "_.input")], vec![]);
			let package = package.unwrap();

			let rendered = package.compile(Some(input.clone()), None);

			assert_eq!(rendered.unwrap(), input);
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
					("templates/input.jsonnet", "_.input"),
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
					("templates/input.jsonnet", "_.input"),
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
					("vendor/ksonnet/ksonnet.beta.4/k8s.libjsonnet", "_.input"),
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
					("vendor/ksonnet/ksonnet.beta.4/k8s.libjsonnet", "_.input"),
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
				vec![("templates/main.jsonnet", "_.files('database.toml')")],
				vec![],
			);
			let package = package.unwrap();
			let input = package.example.clone().unwrap();
			let template = helpers::template(&Fixture::contents("kcp/files/database.toml"), &input);
			let rendered = package.compile(Some(input), None);

			assert_eq!(rendered.unwrap(), Value::String(template));
		}

		#[test]
		fn renders_multiple_templates() {
			let (package, _dir) = package(
				vec![("templates/main.jsonnet", "_.files('**/*.toml')")],
				vec![],
			);
			let package = package.unwrap();
			let input = package.example.clone().unwrap();

			let db_template =
				helpers::template(&Fixture::contents("kcp/files/database.toml"), &input);
			let evt_template =
				helpers::template(&Fixture::contents("kcp/files/events/settings.toml"), &input);

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
				vec![("templates/main.jsonnet", "_.files('invalid.ini')")],
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
				vec![("templates/main.jsonnet", "_.files('no-params.txt')")],
				vec!["example.json", "schema.json"],
			);
			let package = package.unwrap();

			let template = helpers::template(
				&Fixture::contents("kcp/files/no-params.txt"),
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
				vec![("templates/main.jsonnet", "_.files('*.json')")],
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
			let (package, _dir) = package(vec![("templates/main.jsonnet", "_.name")], vec![]);
			let package = package.unwrap();

			let json = format!(r#""{}-{}""#, release.name, package.spec.name);
			let rendered = compile_with_example(package, Some(release));

			let result = helpers::json(&json);

			assert_eq!(rendered.unwrap(), result);
		}

		#[test]
		fn is_injected_on_global() {
			let release = Release {
				name: String::from("rc"),
			};
			let (package, _dir) = package(vec![("templates/main.jsonnet", "_.release")], vec![]);
			let package = package.unwrap();

			let json = format!(r#"{{ "name": "{0}" }}"#, release.name);
			let rendered = compile_with_example(package, Some(release));

			let result = helpers::json(&json);

			assert_eq!(rendered.unwrap(), result);
		}
	}

	mod package {
		use super::*;

		#[test]
		fn is_injected_on_global() {
			let (package, _dir) = package(vec![("templates/main.jsonnet", "_.package")], vec![]);
			let package = package.unwrap();

			let json = format!(
				r#"{{ "name": "{}", "version": "{}" }}"#,
				package.spec.name, package.spec.version
			);
			let rendered = compile_with_example(package, None);

			let result = helpers::json(&json);

			assert_eq!(rendered.unwrap(), result);
		}

		#[test]
		fn is_default_installation_name() {
			let (package, _dir) = package(vec![("templates/main.jsonnet", "_.name")], vec![]);
			let package = package.unwrap();

			let json = format!(r#""{}""#, package.spec.name);
			let rendered = compile_with_example(package, None);

			let result = helpers::json(&json);

			assert_eq!(rendered.unwrap(), result);
		}
	}

	mod subpackage {
		use super::*;

		fn subpackage(dir: &TempDir, name: &str, with: Vec<(&str, &str)>, without: Vec<&str>) {
			use fs_extra::dir::{create_all, move_dir, CopyOptions};
			let (_package, source) = package(with, without);

			let path = dir.path().join("vendor").join(name);
			create_all(&path, true)
				.expect("Failed to create subpackage directory as vendor of parent");

			let mut options = CopyOptions::new();
			options.content_only = true;
			move_dir(source.into_path(), path, &options)
				.expect("Failed to move subpackage into parent");
		}

		#[test]
		fn are_rendered_with_include() {
			let (root, dir) = package(
				vec![("templates/main.jsonnet", "_.include('sub', _.input)")],
				vec![],
			);
			subpackage(
				&dir,
				"sub",
				vec![("templates/main.jsonnet", "_.input")],
				vec![],
			);
			let package = root.unwrap();
			let rendered = compile_with_example(package, None);

			let result = helpers::json(&Fixture::contents("kcp/example.json"));

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
					"_.include('sub', { database: null })",
				)],
				vec![],
			);
			let _archive = subpackage(
				&dir,
				"sub",
				vec![("templates/main.jsonnet", "_.input")],
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
				vec![("templates/main.jsonnet", "_.include('sub', _.input)")],
				vec![],
			);
			let _archive = subpackage(
				&dir,
				"sub",
				vec![("templates/main.jsonnet", "_.release")],
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
				vec![("templates/main.jsonnet", "_.include('dep', _.input)")],
				vec![],
			);
			subpackage(
				&dir,
				"dep",
				vec![("templates/main.jsonnet", "_.include('transient', _.input)")],
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

			let result = helpers::json(contents);

			assert_eq!(rendered.unwrap(), result);
		}
	}
}
