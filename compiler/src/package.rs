pub mod error;
pub mod schema;
pub mod spec;

use self::error::{Error, Result};
use self::schema::Schema;
use self::spec::Spec;
use flate2::read::GzDecoder;
use std::fs::File;
use std::path::PathBuf;
use tar::Archive;
use tempfile::TempDir;

const SCHEMA_FILE: &str = "values.schema.json";
const SPEC_FILE: &str = "kcp.json";

pub struct Package {
	pub root: PathBuf,
	pub spec: Spec,
	pub schema: Option<Schema>,
}

impl Package {
	pub fn from_path(root: PathBuf) -> Result<Self> {
		let root = match root.extension() {
			None => root,
			Some(_) => decompress(root)?,
		};

		let mut spec = root.clone();
		spec.push(SPEC_FILE);
		let spec = Spec::from_path(spec)?;

		let mut schema = root.clone();
		schema.push(SCHEMA_FILE);
		let schema = match Schema::from_path(schema) {
			Ok(schema) => Some(schema),
			Err(Error::NoSchema) => None,
			Err(err) => return Err(err),
		};

		Ok(Package { root, spec, schema })
	}
}

fn decompress(archive: PathBuf) -> Result<PathBuf> {
	let ext = archive.extension().unwrap().to_str();
	if ext != Some("tgz") {
		return Err(Error::InvalidFormat);
	}

	let dir = TempDir::new()
		.expect("Unable to create temporary directory to unpack your KCP")
		.into_path();

	let kcp = File::open(archive).expect("Unable to read KCP archive");
	let tar = GzDecoder::new(kcp);
	let mut archive = Archive::new(tar);
	archive
		.unpack(dir.to_str().unwrap())
		.expect("Unable to extract your KCP archive");

	Ok(dir)
}
