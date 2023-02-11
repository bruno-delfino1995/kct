use std::path::{Path, PathBuf};

use once_cell::sync::Lazy;
use regex::Regex;

#[derive(Default)]
pub struct Filter {
	pub only: Vec<PathBuf>,
	pub except: Vec<PathBuf>,
}

impl Filter {
	pub fn pass(&self, path: &Path) -> bool {
		let allow = self.only.iter().any(|allow| path.starts_with(allow));

		let disallow = self
			.except
			.iter()
			.any(|disallow| path.starts_with(disallow));

		(allow || self.only.is_empty()) && !disallow
	}
}

static RE: Lazy<Regex> =
	Lazy::new(|| Regex::new(r"(?i)^[a-z0-9]$|^[a-z0-9][a-z0-9-]*[a-z0-9]$").unwrap());

pub fn is_valid(path: &str) -> bool {
	RE.is_match(path)
}
