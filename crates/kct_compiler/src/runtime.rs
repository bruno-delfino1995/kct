use crate::extension::Plugins;
use crate::{Compiler, Context, Target};

use std::convert::From;

#[derive(Clone)]
pub struct Runtime {
	pub context: Context,
	pub workspace: Target,
	pub plugins: Plugins,
}

impl From<&Compiler> for Runtime {
	fn from(compiler: &Compiler) -> Self {
		Runtime {
			context: compiler.context.clone(),
			workspace: compiler.workspace.clone(),
			plugins: compiler.plugins.clone(),
		}
	}
}
