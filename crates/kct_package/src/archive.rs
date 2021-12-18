use flate2::write::GzEncoder;
use flate2::Compression;
use std::fs::File;
use std::path::{Path, PathBuf};
use tar::Builder;

const EXTENSION: &str = "tgz";

pub fn archive(name: &str, source: &Path, below: &Path) -> Result<PathBuf, String> {
	let mut target = below.to_path_buf();
	target.push(format!("{}.{}", name, EXTENSION));

	let file = File::create(target.clone()).map_err(|err| err.to_string())?;
	let enc = GzEncoder::new(file, Compression::default());
	let mut tar = Builder::new(enc);

	tar.append_dir_all("", source)
		.map_err(|err| err.to_string())?;

	Ok(target)
}
