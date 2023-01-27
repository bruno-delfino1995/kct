use crate::error::{Error, Result};

use std::convert::TryFrom;
use std::path::PathBuf;

use kct_helper::io;
use semver::Version;
use serde_json::Value;

#[derive(Debug, Clone)]
pub struct Spec {
	pub name: String,
	pub version: Version,
}

impl TryFrom<PathBuf> for Spec {
	type Error = Error;

	fn try_from(path: PathBuf) -> Result<Spec> {
		match io::from_file(&path) {
			Ok(contents) => {
				let json: Value =
					serde_json::from_str(&contents).map_err(|_err| Error::InvalidSpec)?;
				let name = json
					.get("name")
					.and_then(|v| v.as_str())
					.map(|v| Ok(String::from(v)))
					.unwrap_or(Err(Error::InvalidSpec))?;

				let version = json
					.get("version")
					.and_then(|v| v.as_str())
					.map(|v| Version::parse(v).map_err(|_err| Error::InvalidSpec))
					.unwrap_or(Err(Error::InvalidSpec))?;

				Ok(Spec { name, version })
			}
			_ => Err(Error::InvalidSpec),
		}
	}
}
