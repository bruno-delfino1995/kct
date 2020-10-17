use super::{Error, Result};
use kct_helper::io;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Deserialize, Serialize)]
pub struct Spec {
	pub name: String,
}

impl Spec {
	pub fn from_path(path: PathBuf) -> Result<Spec> {
		match io::from_file(&path) {
			Ok(contents) => {
				let spec: Spec =
					serde_json::from_str(&contents).map_err(|_err| Error::InvalidSpec)?;

				Ok(spec)
			}
			_ => Err(Error::InvalidSpec),
		}
	}
}
