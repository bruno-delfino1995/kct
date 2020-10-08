mod fixtures;

use fixtures::Fixture;
use package::{error::Error, Package, Release};
use serde_json::{Map, Value};
use std::path::PathBuf;
use tempfile::TempDir;

fn default_package() -> Package {
	let dir = Fixture::dir(&["kcp.json", "values.schema.json", "valid.jsonnet"]);

	// TODO: Find a way to drop from TempDir for cleanup after test is done
	Package::from_path(dir.into_path()).unwrap()
}

mod from_path {
	use super::*;

	#[test]
	fn can_be_created() {
		default_package();
	}

	#[test]
	fn need_spec() {
		let dir = Fixture::dir(&["values.schema.json", "valid.jsonnet"]);
		let package = Package::from_path(PathBuf::from(dir.path()));

		assert!(package.is_err());
		assert_eq!(package.unwrap_err(), Error::NoSpec)
	}

	#[test]
	fn from_archive() {
		let cwd = TempDir::new().unwrap();
		let dir = Fixture::dir(&["kcp.json", "values.schema.json", "valid.jsonnet"]);
		let archive = Package::from_path(PathBuf::from(dir.path()))
			.unwrap()
			.archive(&PathBuf::from(cwd.path()))
			.unwrap();

		let package = Package::from_path(archive);

		assert!(package.is_ok())
	}
}

mod archive {
	use super::*;

	#[test]
	fn creates_a_file_on_provided_dir() {
		let cwd = TempDir::new().unwrap();
		let package = default_package();

		let compressed = package.archive(&PathBuf::from(cwd.path()));

		assert!(compressed.is_ok());
		assert!(compressed.unwrap().starts_with(cwd.path()));
	}

	#[test]
	fn creates_archive_with_spec_name() {
		let cwd = TempDir::new().unwrap();
		let package = default_package();
		let name = package.spec.name.clone();

		let compressed = package.archive(&PathBuf::from(cwd.path()));

		assert!(compressed.is_ok());
		assert_eq!(
			compressed.unwrap().file_stem().unwrap().to_str().unwrap(),
			name
		);
	}

	#[test]
	fn can_be_compiled_after_archived() {
		let cwd = TempDir::new().unwrap();
		let package = default_package();
		let values = Some(Fixture::values("values.json"));

		let compressed = package.archive(&PathBuf::from(cwd.path())).unwrap();
		let package = Package::from_path(compressed).unwrap();
		let compiled = package.compile(values, None);

		assert!(compiled.is_ok());
	}
}

mod compile {
	use super::*;

	mod preconditions {
		use super::*;

		#[test]
		fn requests_values() {
			let package = Fixture::package(&["valid.jsonnet"], Some("values.schema.json"));

			let rendered = package.compile(None, None);
			assert_eq!(rendered.unwrap_err(), Error::NoValues);
		}

		#[test]
		fn request_schema() {
			let package = Fixture::package(&["valid.jsonnet"], None);
			let values = Some(Fixture::values("values.json"));

			let rendered = package.compile(values, None);
			assert_eq!(rendered.unwrap_err(), Error::NoSchema);
		}
	}

	mod values {
		use super::*;

		#[test]
		fn renders_with_null() {
			let package = Fixture::package(&["valid.jsonnet"], None);
			let rendered = package.compile(None, None);

			let json = r#"{ "values": null }"#;
			let result: Value = serde_json::from_str(json).unwrap();
			assert_eq!(rendered.unwrap(), result);
		}

		#[test]
		fn renders_with_value() {
			let package = Fixture::package(&["valid.jsonnet"], Some("values.schema.json"));
			let values = Some(Fixture::values("values.json"));

			let rendered = package.compile(values.clone(), None);

			let json = format!(r#"{{ "values": {0} }}"#, values.unwrap());
			let result: Value = serde_json::from_str(&json).unwrap();
			assert_eq!(rendered.unwrap(), result);
		}
	}

	mod jsonnet {
		use super::*;

		#[test]
		#[should_panic(expected = "manifest function")]
		fn disallows_top_level_functions() {
			let package = Fixture::package(&["function.jsonnet"], None);

			let rendered = package.compile(None, None).unwrap_err();

			match rendered {
				Error::RenderIssue(err) => panic!(err),
				_ => panic!("It should be a render issue!"),
			}
		}

		#[test]
		fn renders_imports() {
			let package = Fixture::package(
				&["import.jsonnet", "valid.jsonnet"],
				Some("values.schema.json"),
			);
			let values = Some(Fixture::values("values.json"));

			let rendered = package.compile(values.clone(), None);

			let json = format!(r#"{{ "imported": {{ "values": {0} }} }}"#, values.unwrap());
			let result: Value = serde_json::from_str(&json).unwrap();
			assert_eq!(rendered.unwrap(), result);
		}
	}

	mod file_templates {
		use super::*;

		#[test]
		fn renders_templates() {
			let package = Fixture::package(
				&["with-template.jsonnet", "files/database.toml"],
				Some("values.schema.json"),
			);
			let values = Some(Fixture::values("values.json"));
			let template = Fixture::template("database.toml", values.clone().unwrap());

			let rendered = package.compile(values.clone(), None);

			let expected = {
				let mut map = Map::<String, Value>::new();
				map.insert(String::from("values"), values.unwrap());
				map.insert(String::from("settings"), Value::String(template));
				Value::Object(map)
			};

			assert_eq!(rendered.unwrap(), expected);
		}

		#[test]
		fn renders_multiple_templates() {
			let package = Fixture::package(
				&[
					"with-multiple-templates.jsonnet",
					"files/database.toml",
					"files/events/settings.toml",
				],
				Some("values.schema.json"),
			);
			let values = Some(Fixture::values("values.json"));
			let db_template = Fixture::template("database.toml", values.clone().unwrap());
			let evt_template = Fixture::template("events/settings.toml", values.clone().unwrap());

			let rendered = package.compile(values.clone(), None);

			let expected = {
				let mut map = Map::<String, Value>::new();
				map.insert(String::from("values"), values.unwrap());
				map.insert(
					String::from("settings"),
					Value::Array(vec![
						Value::String(db_template),
						Value::String(evt_template),
					]),
				);

				Value::Object(map)
			};

			assert_eq!(rendered.unwrap(), expected);
		}

		#[test]
		#[should_panic(expected = "Unable to compile templates")]
		fn fails_on_invalid_templates() {
			let package = Fixture::package(
				&["with-invalid-template.jsonnet", "files/invalid.ini"],
				Some("values.schema.json"),
			);
			let values = Some(Fixture::values("values.json"));

			let rendered = package.compile(values, None).unwrap_err();

			match rendered {
				Error::RenderIssue(err) => panic!(err),
				_ => panic!("It should be a render issue!"),
			}
		}

		#[test]
		fn compiles_templates_with_empty_values() {
			let package =
				Fixture::package(&["plain-template.jsonnet", "files/no-params.txt"], None);
			let template = Fixture::template("no-params.txt", Value::Object(Map::new()));

			let rendered = package.compile(None, None);

			let expected = {
				let mut map = Map::<String, Value>::new();
				map.insert(String::from("settings"), Value::String(template));

				Value::Object(map)
			};

			assert_eq!(rendered.unwrap(), expected);
		}

		#[test]
		#[should_panic(expected = "No files folder to search for templates")]
		fn fails_on_empty_templates_folder() {
			let package = Fixture::package(&["plain-template.jsonnet"], None);

			let rendered = package.compile(None, None).unwrap_err();

			match rendered {
				Error::RenderIssue(err) => panic!(err),
				_ => panic!("It should be a render issue!"),
			}
		}

		#[test]
		#[should_panic(expected = "No template found for glob")]
		fn fails_on_not_found_template() {
			let package =
				Fixture::package(&["plain-template.jsonnet", "files/database.toml"], None);

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
			let package = Fixture::package(&["package.jsonnet"], None);
			let package_name = package.spec.name.clone();

			let rendered = package.compile(
				None,
				Some(Release {
					name: String::from(release_name),
				}),
			);

			let json = format!(
				r#"{{ "package": {{ "name": "{0}", "fullName": "{1}" }} }}"#,
				package_name,
				format!("{}-{}", release_name, package_name)
			);
			let result: Value = serde_json::from_str(&json).unwrap();

			assert_eq!(rendered.unwrap(), result);
		}

		#[test]
		fn is_injected_on_global() {
			let release = Release {
				name: String::from("rc"),
			};
			let package = Fixture::package(&["release.jsonnet"], None);

			let json = format!(r#"{{ "release": {{ "name": "{0}" }} }}"#, release.name);
			let rendered = package.compile(None, Some(release));

			let result: Value = serde_json::from_str(&json).unwrap();

			assert_eq!(rendered.unwrap(), result);
		}
	}
}
