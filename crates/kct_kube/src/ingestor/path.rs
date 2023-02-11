use std::path::{Path, PathBuf};

use lazy_static::lazy_static;
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

pub fn is_valid(path: &str) -> bool {
	lazy_static! {
		static ref PATTERN: Regex =
			Regex::new(r"(?i)^[a-z0-9]$|^[a-z0-9][a-z0-9-]*[a-z0-9]$").unwrap();
	}

	PATTERN.is_match(path)
}
