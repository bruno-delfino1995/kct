use std::fs;
use std::path::Path;

pub fn rm(at: &Path) {
	if at.is_dir() {
		fs::remove_dir_all(&at)
			.unwrap_or_else(|_| panic!("Unable to remove dir at: {}", &at.display()));
	} else {
		fs::remove_file(&at)
			.unwrap_or_else(|_| panic!("Unable to delete file at: {}", &at.display()));
	}
}
