mod fixtures;
mod helpers;

use fixtures::Fixture;
use kct_helper::json;
use kct_package::{error::Error, Package, Release};
use serde_json::{Map, Value};
use std::fs;
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

	let package = Package::from_path(PathBuf::from(dir.path()));

	(package, dir)
}

mod from_path {
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

		let package = Package::from_path(archive);

		assert!(package.is_ok())
	}

	#[test]
	fn need_spec() {
		let (package, _dir) = package(vec![], vec!["kcp.json"]);

		assert!(package.is_err());
		assert_eq!(package.unwrap_err(), Error::NoSpec)
	}

	#[test]
	fn requests_values_for_schema() {
		let (package, _dir) = package(vec![], vec!["values.json"]);

		assert!(package.is_err());
		assert_eq!(package.unwrap_err(), Error::NoValues)
	}

	#[test]
	fn request_schema_for_values() {
		let (package, _dir) = package(vec![], vec!["values.schema.json"]);

		assert!(package.is_err());
		assert_eq!(package.unwrap_err(), Error::NoSchema)
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

		let values = helpers::values(&Fixture::contents("kcp/values.json"));

		let compressed = package.archive(&PathBuf::from(cwd.path())).unwrap();
		let package = Package::from_path(compressed).unwrap();
		let compiled = package.compile(Some(values), None);

		assert!(compiled.is_ok());
	}
}

mod compile {
	use super::*;

	mod values {
		use super::*;

		#[test]
		fn renders_with_null() {
			let (package, _dir) = package(
				vec![("templates/main.jsonnet", "_.values")],
				vec!["values.json", "values.schema.json"],
			);
			let package = package.unwrap();

			let rendered = package.compile(None, None);

			assert_eq!(rendered.unwrap(), Value::Null);
		}

		#[test]
		fn renders_with_default_values() {
			let (package, _dir) = package(vec![("templates/main.jsonnet", "_.values")], vec![]);
			let package = package.unwrap();

			let values = helpers::values(&Fixture::contents("kcp/values.json"));

			let rendered = package.compile(None, None);

			assert_eq!(rendered.unwrap(), values);
		}

		#[test]
		fn merges_values_with_defaults() {
			let defaults = helpers::values(&Fixture::contents("kcp/values.json"));
			let values: Value = helpers::values(
				r#"{ "database": { "port": 5432, "credentials": { "user": "admin", "pass": "admin" } } }"#,
			);
			let merged = {
				let mut merged = defaults;
				json::merge(&mut merged, &values);
				merged
			};

			let (package, _dir) = package(vec![("templates/main.jsonnet", "_.values")], vec![]);
			let package = package.unwrap();

			let rendered = package.compile(Some(values), None);

			assert_eq!(rendered.unwrap(), merged);
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
					"function(values = null, files = null) { values: values }",
				)],
				vec![],
			);
			let package = package.unwrap();

			let rendered = package.compile(None, None).unwrap_err();

			match rendered {
				Error::RenderIssue(err) => panic!(err),
				_ => panic!("It should be a render issue!"),
			}
		}

		#[test]
		fn renders_imports() {
			let (package, _dir) = package(
				vec![
					(
						"templates/main.jsonnet",
						"local valid = import './values/entry.jsonnet'; valid",
					),
					(
						"templates/values/entry.jsonnet",
						"import '../values.jsonnet'",
					),
					("templates/values.jsonnet", "_.values"),
				],
				vec![],
			);
			let package = package.unwrap();
			let values = package.values.clone().unwrap();

			let rendered = package.compile(None, None);

			assert_eq!(rendered.unwrap(), values);
		}

		#[test]
		#[should_panic(expected = "can't resolve values.jsonnet")]
		fn doesnt_include_templates_on_imports() {
			let (package, _dir) = package(
				vec![
					(
						"templates/main.jsonnet",
						"local valid = import './values/entry.jsonnet'; valid",
					),
					("templates/values/entry.jsonnet", "import 'values.jsonnet'"),
					("templates/values.jsonnet", "_.values"),
				],
				vec![],
			);
			let package = package.unwrap();

			let rendered = package.compile(None, None).unwrap_err();

			match rendered {
				Error::RenderIssue(err) => panic!(err),
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
					("vendor/ksonnet/ksonnet.beta.4/k8s.libjsonnet", "_.values"),
				],
				vec![],
			);
			let package = package.unwrap();
			let values = package.values.clone().unwrap();

			let rendered = package.compile(None, None);

			assert_eq!(rendered.unwrap(), values);
		}

		#[test]
		fn includes_lib_for_aliasing() {
			let (package, _dir) = package(
				vec![
					(
						"templates/main.jsonnet",
						"local valid = import 'k.libjsonnet'; valid",
					),
					("vendor/ksonnet/ksonnet.beta.4/k8s.libjsonnet", "_.values"),
					(
						"lib/k.libjsonnet",
						"import 'ksonnet/ksonnet.beta.4/k8s.libjsonnet'",
					),
				],
				vec![],
			);
			let package = package.unwrap();
			let values = package.values.clone().unwrap();

			let rendered = package.compile(None, None);

			assert_eq!(rendered.unwrap(), values);
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
			let values = package.values.clone().unwrap();

			let template =
				helpers::template(&Fixture::contents("kcp/files/database.toml"), &values);
			let rendered = package.compile(None, None);

			assert_eq!(rendered.unwrap(), Value::String(template));
		}

		#[test]
		fn renders_multiple_templates() {
			let (package, _dir) = package(
				vec![("templates/main.jsonnet", "_.files('**/*.toml')")],
				vec![],
			);
			let package = package.unwrap();
			let values = package.values.clone().unwrap();

			let db_template =
				helpers::template(&Fixture::contents("kcp/files/database.toml"), &values);
			let evt_template = helpers::template(
				&Fixture::contents("kcp/files/events/settings.toml"),
				&values,
			);

			let rendered = package.compile(None, None);

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

			let rendered = package.compile(None, None).unwrap_err();

			match rendered {
				Error::RenderIssue(err) => panic!(err),
				_ => panic!("It should be a render issue!"),
			}
		}

		#[test]
		fn compiles_templates_with_empty_values() {
			let (package, _dir) = package(
				vec![("templates/main.jsonnet", "_.files('no-params.txt')")],
				vec!["values.json", "values.schema.json"],
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

			let rendered = package.compile(None, None).unwrap_err();

			match rendered {
				Error::RenderIssue(err) => panic!(err),
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

			let rendered = package.compile(None, None).unwrap_err();

			match rendered {
				Error::RenderIssue(err) => panic!(err),
				_ => panic!("It should be a render issue!"),
			}
		}
	}

	mod release {
		use super::*;

		#[test]
		fn prefixes_package_name() {
			let release_name = "rc";
			let (package, _dir) = package(vec![("templates/main.jsonnet", "_.package")], vec![]);
			let package = package.unwrap();
			let expected = format!("{}-{}", release_name, package.spec.name);

			let rendered = package
				.compile(
					None,
					Some(Release {
						name: String::from(release_name),
					}),
				)
				.unwrap();
			let actual = rendered.get("fullName").unwrap().as_str().unwrap();

			assert_eq!(actual, expected);
		}

		#[test]
		fn is_injected_on_global() {
			let release = Release {
				name: String::from("rc"),
			};
			let (package, _dir) = package(vec![("templates/main.jsonnet", "_.release")], vec![]);
			let package = package.unwrap();

			let json = format!(r#"{{ "name": "{0}" }}"#, release.name);
			let rendered = package.compile(None, Some(release));

			let result = helpers::values(&json);

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
				r#"{{ "name": "{0}", "fullName": "{1}", "version": "{2}" }}"#,
				package.spec.name, package.spec.name, package.spec.version
			);
			let rendered = package.compile(None, None);

			let result = helpers::values(&json);

			assert_eq!(rendered.unwrap(), result);
		}
	}

	mod subpackage {
		use super::*;

		fn subpackage(
			dir: &TempDir,
			name: &str,
			with: Vec<(&str, &str)>,
			without: Vec<&str>,
		) -> PathBuf {
			let (package, _dir) = package(with, without);
			let package = package.unwrap();

			let archive = {
				let path = dir.path().join("kcps");
				fs::create_dir(&path).unwrap();

				package.archive(&path).unwrap()
			};

			let at = {
				let mut sub = archive.clone();
				sub.set_file_name(name);
				sub.set_extension("tgz");

				sub
			};

			fs::rename(&archive, &at).unwrap();

			at
		}

		#[test]
		fn are_rendered_with_include() {
			let (root, dir) = package(
				vec![("templates/main.jsonnet", "_.include('sub', _.values)")],
				vec![],
			);
			let _archive = subpackage(
				&dir,
				"sub",
				vec![("templates/main.jsonnet", "_.values")],
				vec![],
			);
			let package = root.unwrap();

			let rendered = package.compile(None, None);

			let result = helpers::values(&Fixture::contents("kcp/values.json"));

			assert_eq!(rendered.unwrap(), result);
		}

		// NOTE: As subpackages are supposed to be archived packages, we don't
		// need to validate all the same cases as we do for packages. This test
		// is usefull to guarantee that we pass the correct values while
		// compiling the subpackage, not to check the realm of invalid packages
		#[test]
		#[should_panic(expected = "values provided don't match the schema")]
		fn validate_values() {
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
				vec![("templates/main.jsonnet", "_.values")],
				vec![],
			);
			let package = root.unwrap();

			let rendered = package.compile(None, None).unwrap_err();

			match rendered {
				Error::RenderIssue(err) => panic!(err),
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
				vec![("templates/main.jsonnet", "_.include('sub', _.values)")],
				vec![],
			);
			let _archive = subpackage(
				&dir,
				"sub",
				vec![("templates/main.jsonnet", "_.release")],
				vec![],
			);
			let package = root.unwrap();

			let rendered = package.compile(None, Some(release)).unwrap();

			let actual = rendered.get("name").unwrap().as_str().unwrap();
			assert_eq!(actual, name);
		}
	}
}
