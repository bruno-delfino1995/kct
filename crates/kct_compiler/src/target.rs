use std::convert::From;
use std::path::{Path, PathBuf};

pub struct Target {
	dir: PathBuf,
	main: PathBuf,
	lib: PathBuf,
}

impl Target {
	pub fn dir(&self) -> &Path {
		&self.dir
	}

	pub fn main(&self) -> &Path {
		&self.main
	}

	pub fn lib(&self) -> &Path {
		&self.lib
	}
}

#[derive(Default)]
pub struct TargetBuilder {
	dir: Option<PathBuf>,
	main: Option<PathBuf>,
	lib: Option<PathBuf>,
}

impl TargetBuilder {
	pub fn dir(mut self, root: PathBuf) -> Self {
		match self.dir {
			Some(_) => self,
			None => {
				self.dir = Some(root);

				self
			}
		}
	}

	pub fn main(mut self, main: PathBuf) -> Self {
		match self.main {
			Some(_) => self,
			None => {
				self.main = Some(main);

				self
			}
		}
	}

	pub fn build(self) -> Result<Target, String> {
		let dir = self.dir.ok_or_else(|| String::from("dir is required"))?;
		let main = self.main.ok_or_else(|| String::from("main is required"))?;
		let lib = self.lib.unwrap_or_else(|| default_lib(&dir));

		Ok(Target { dir, main, lib })
	}
}

fn default_lib(root: &Path) -> PathBuf {
	let mut path = root.to_path_buf();
	path.push("lib");

	path
}
