use crate::Release;

use std::convert::From;
use std::path::{Path, PathBuf};
use std::rc::Rc;

struct Internal {
	root: PathBuf,
	release: Option<Release>,
	vendor: PathBuf,
}

#[derive(Clone)]
pub struct Context(Rc<Internal>);

impl Context {
	fn default_vendor(root: &Path) -> PathBuf {
		let mut path = root.to_path_buf();
		path.push("vendor");

		path
	}
}

impl Context {
	pub fn root(&self) -> &Path {
		&self.0.root
	}

	pub fn release(&self) -> &Option<Release> {
		&self.0.release
	}

	pub fn vendor(&self) -> &Path {
		&self.0.vendor
	}
}

#[derive(Default)]
pub struct ContextBuilder {
	root: Option<PathBuf>,
	release: Option<Release>,
	vendor: Option<PathBuf>,
}

impl ContextBuilder {
	pub fn root(mut self, root: PathBuf) -> ContextBuilder {
		match self.root {
			Some(_) => self,
			None => {
				self.root = Some(root);

				self
			}
		}
	}

	pub fn release(mut self, release: Option<Release>) -> ContextBuilder {
		match self.release {
			Some(_) => self,
			None => {
				self.release = release;

				self
			}
		}
	}

	pub fn build(self) -> Result<Context, String> {
		let root = self.root.ok_or_else(|| String::from("root is required"))?;
		let release = self.release;
		let vendor = self
			.vendor
			.unwrap_or_else(|| Context::default_vendor(&root));

		let internal = Internal {
			root,
			release,
			vendor,
		};

		Ok(Context(Rc::new(internal)))
	}
}
