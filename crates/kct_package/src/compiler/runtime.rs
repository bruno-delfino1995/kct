use super::property::{Name, Output};
use super::workspace::Workspace;
use super::{Compiler, Context};

use std::collections::HashMap;
use std::convert::From;
use std::rc::Rc;

#[derive(Clone)]
pub struct Runtime {
	pub context: Context,
	pub workspace: Workspace,
	pub properties: HashMap<Name, Rc<Output>>,
}

impl From<&Compiler> for Runtime {
	fn from(compiler: &Compiler) -> Self {
		Runtime {
			context: compiler.context.clone(),
			workspace: compiler.workspace.clone(),
			properties: compiler.properties.clone(),
		}
	}
}
