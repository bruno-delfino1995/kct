use package::{schema::Schema, spec::Spec, Package};
use serde_json::Value;
use std::env;
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;
use tera::{Context, Tera};

pub struct Fixture {}

impl Fixture {
	fn root() -> PathBuf {
		let root_dir = &env::var("CARGO_MANIFEST_DIR").expect("$CARGO_MANIFEST_DIR");

		let mut source = PathBuf::from(root_dir);
		source.push("tests/fixtures");
		source
	}

	pub fn dir(with: &[&str]) -> TempDir {
		let sources = with.iter().map(|name| Self::path(name));

		let tempdir = TempDir::new().unwrap();
		let dir = tempdir.path();
		let targets = with.iter().map(|name| {
			let mut target = PathBuf::from(dir);
			target.push(name);
			target
		});

		make_dirs(
			&PathBuf::from(tempdir.path()),
			&with.iter().map(PathBuf::from).collect::<Vec<PathBuf>>(),
		);

		for (source, target) in sources.zip(targets) {
			fs::copy(source, target).unwrap();
		}

		tempdir
	}

	pub fn path(name: &str) -> PathBuf {
		let mut source = Self::root();
		source.push(name);

		source
	}

	pub fn package(templates: &[&str], schema: Option<&str>) -> Package {
		let brownfield = Self::dir(&templates);
		let root = PathBuf::from(brownfield.path());

		let mut main = root.clone();
		main.push(templates[0]);
		let spec = Spec {
			main,
			name: String::from("fixture"),
		};

		let schema = schema
			.map(Self::path)
			.map(fs::read_to_string)
			.map(Result::unwrap)
			.map(|contents| serde_json::from_str(&contents))
			.map(Result::unwrap)
			.map(Schema::new)
			.map(Result::unwrap);

		Package {
			root,
			spec,
			schema,
			brownfield: Some(brownfield),
		}
	}

	pub fn values(name: &str) -> Value {
		let path = Self::path(name);

		fs::read_to_string(path)
			.map(|contents| serde_json::from_str(&contents))
			.map(Result::unwrap)
			.unwrap()
	}

	pub fn template(name: &str, values: Value) -> String {
		let mut template = Self::root();
		template.push("files");
		template.push(name);

		let contents = fs::read_to_string(template).unwrap();
		let context = Context::from_serialize(values).unwrap();

		Tera::one_off(&contents, &context, true).unwrap()
	}
}

fn make_dirs(at: &PathBuf, paths: &[PathBuf]) {
	for path in paths {
		let mut target = at.clone();
		let mut dir = path.clone();
		dir.pop();

		target.push(dir);
		fs::create_dir_all(target).unwrap()
	}
}
