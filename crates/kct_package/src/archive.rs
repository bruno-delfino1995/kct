use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use flate2::Compression;
use std::fs::File;
use std::path::{Path, PathBuf};
use tar::{Archive, Builder};

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

pub fn unarchive(archive: &Path, dest: &Path) -> Result<(), String> {
	let ext = archive.extension().unwrap().to_str();
	if ext != Some(EXTENSION) {
		return Err(String::from("Package is not a .tgz"));
	}

	let kcp = File::open(archive).expect("Unable to read KCP archive");
	let tar = GzDecoder::new(kcp);
	let mut archive = Archive::new(tar);
	archive
		.unpack(dest.to_str().unwrap())
		.expect("Unable to extract your KCP archive");

	Ok(())
}
