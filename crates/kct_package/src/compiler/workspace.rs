use std::convert::From;
use std::path::{Path, PathBuf};

#[derive(Clone, Default)]
pub struct Workspace {
	root: PathBuf,
	entrypoint: PathBuf,
	lib: PathBuf,
	vendor: PathBuf,
}

impl Workspace {
	fn default_vendor(root: &Path) -> PathBuf {
		let mut path = root.to_path_buf();
		path.push("vendor");

		path
	}

	fn default_lib(root: &Path) -> PathBuf {
		let mut path = root.to_path_buf();
		path.push("lib");

		path
	}
}

impl Workspace {
	pub fn root(&self) -> &Path {
		&self.root
	}

	pub fn entrypoint(&self) -> &Path {
		&self.entrypoint
	}

	pub fn lib(&self) -> &Path {
		&self.lib
	}

	pub fn vendor(&self) -> &Path {
		&self.vendor
	}
}

#[derive(Default)]
pub struct WorkspaceBuilder {
	root: Option<PathBuf>,
	entrypoint: Option<PathBuf>,
	lib: Option<PathBuf>,
	vendor: Option<PathBuf>,
}

impl WorkspaceBuilder {
	pub fn root(mut self, root: PathBuf) -> WorkspaceBuilder {
		match self.root {
			Some(_) => self,
			None => {
				self.root = Some(root);

				self
			}
		}
	}

	pub fn entrypoint(mut self, entrypoint: PathBuf) -> WorkspaceBuilder {
		match self.entrypoint {
			Some(_) => self,
			None => {
				self.entrypoint = Some(entrypoint);

				self
			}
		}
	}

	#[allow(dead_code)]
	pub fn lib(mut self, lib: PathBuf) -> WorkspaceBuilder {
		match self.lib {
			Some(_) => self,
			None => {
				self.lib = Some(lib);

				self
			}
		}
	}

	pub fn vendor(mut self, vendor: PathBuf) -> WorkspaceBuilder {
		match self.vendor {
			Some(_) => self,
			None => {
				self.vendor = Some(vendor);

				self
			}
		}
	}

	pub fn build(self) -> Result<Workspace, String> {
		let root = self.root.ok_or_else(|| String::from("root is required"))?;
		let entrypoint = self
			.entrypoint
			.ok_or_else(|| String::from("entrypoint is required"))?;
		let lib = self.lib.unwrap_or_else(|| Workspace::default_lib(&root));
		let vendor = self
			.vendor
			.unwrap_or_else(|| Workspace::default_vendor(&root));

		Ok(Workspace {
			root,
			entrypoint,
			lib,
			vendor,
		})
	}
}
