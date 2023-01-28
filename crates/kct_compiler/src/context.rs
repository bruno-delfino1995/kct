use crate::Release;

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
	built: Option<Context>,
	root: Option<PathBuf>,
	release: Option<Release>,
	vendor: Option<PathBuf>,
}

impl ContextBuilder {
	pub fn wrap(ctx: Context) -> Self {
		Self {
			built: Some(ctx),
			..Default::default()
		}
	}

	pub fn root(mut self, root: PathBuf) -> Self {
		if self.built.is_some() {
			return self;
		};

		match self.root {
			Some(_) => self,
			None => {
				self.root = Some(root);

				self
			}
		}
	}

	pub fn release(mut self, release: Option<Release>) -> Self {
		if self.built.is_some() {
			return self;
		};

		match self.release {
			Some(_) => self,
			None => {
				self.release = release;

				self
			}
		}
	}

	pub fn vendor(mut self, vendor: PathBuf) -> Self {
		if self.built.is_some() {
			return self;
		};

		match self.vendor {
			Some(_) => self,
			None => {
				self.vendor = Some(vendor);

				self
			}
		}
	}

	pub fn build(self) -> Result<Context, String> {
		if let Some(built) = self.built {
			return Ok(built);
		}

		let root = self.root.ok_or_else(|| String::from("root is required"))?;
		let release = self.release;
		let vendor = {
			let mut path = root.clone();
			path.push("vendor");

			path
		};

		let internal = Internal {
			root,
			release,
			vendor,
		};

		Ok(Context(Rc::new(internal)))
	}
}
