use std::path::{Path, PathBuf};

use once_cell::sync::Lazy;
use regex::Regex;

/// Blocklist and Allowlist coded into a single filter that allows everything by default. If
/// there's no specific disallow we'll let everything pass, but if there's a specific allow we only
/// permit those allowed. However, if something is allowed and disallowed, we won't let it pass
/// because the blocklist has a higher priority.
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
