use crate::{Compiler, Context, Target};

use std::convert::From;

#[derive(Clone)]
pub struct Runtime {
	pub context: Context,
	pub target: Target,
}

impl From<&Compiler> for Runtime {
	fn from(compiler: &Compiler) -> Self {
		Runtime {
			context: compiler.context.clone(),
			target: compiler.target.clone(),
		}
	}
}
