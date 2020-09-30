mod fixtures;

use fixtures::Fixture;
use package::{error::Error, Package};
use serde_json::Value;
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
		let compiled = package.compile(values);

		assert!(compiled.is_ok());
	}
}

mod compile {
	use super::*;

	#[test]
	fn requests_values() {
		let package = Fixture::package(&["valid.jsonnet"], Some("values.schema.json"));

		let rendered = package.compile(None);
		assert_eq!(rendered.unwrap_err(), Error::NoValues);
	}

	#[test]
	fn request_schema() {
		let package = Fixture::package(&["valid.jsonnet"], None);
		let values = Some(Fixture::values("values.json"));

		let rendered = package.compile(values);
		assert_eq!(rendered.unwrap_err(), Error::NoSchema);
	}

	#[test]
	fn renders_with_null() {
		let package = Fixture::package(&["valid.jsonnet"], None);
		let rendered = package.compile(None);

		let json = r#"{ "values": null }"#;
		let result: Value = serde_json::from_str(json).unwrap();
		assert_eq!(rendered.unwrap(), result);
	}

	#[test]
	fn renders_with_value() {
		let package = Fixture::package(&["valid.jsonnet"], Some("values.schema.json"));
		let values = Some(Fixture::values("values.json"));

		let rendered = package.compile(values.clone());

		let json = format!(r#"{{ "values": {0} }}"#, values.unwrap());
		let result: Value = serde_json::from_str(&json).unwrap();
		assert_eq!(rendered.unwrap(), result);
	}

	#[test]
	fn expects_tla() {
		let package = Fixture::package(&["plain.jsonnet"], None);

		let rendered = package.compile(None).unwrap_err();

		match rendered {
			Error::RenderIssue(_) => (),
			_ => panic!("It should be a render issue!"),
		}
	}

	#[test]
	fn expects_tla_with_values_param() {
		let package = Fixture::package(&["no-param.jsonnet"], None);

		let rendered = package.compile(None).unwrap_err();

		match rendered {
			Error::RenderIssue(_) => (),
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

		let rendered = package.compile(values.clone());

		let json = format!(r#"{{ "imported": {{ "values": {0} }} }}"#, values.unwrap());
		let result: Value = serde_json::from_str(&json).unwrap();
		assert_eq!(rendered.unwrap(), result);
	}
}
