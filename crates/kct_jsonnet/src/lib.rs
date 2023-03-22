mod error;

pub mod property;
pub(crate) mod serde;

use crate::property::Property;
use crate::serde::W;

pub use crate::error::Error;

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::mpsc;
use std::thread;

use anyhow::Result;
use jrsonnet_evaluator::trace::PathResolver;
use jrsonnet_evaluator::{FileImportResolver, State};
use jrsonnet_stdlib::ContextInitializer;
use serde_json::Value;

pub use jrsonnet_evaluator::gc::TraceBox as Track;
pub use jrsonnet_gcmodule::Trace;

const VARS_PREFIX: &str = "kct.io";

pub struct Executable {
	pub vendor: PathBuf,
	pub lib: PathBuf,
	pub main: PathBuf,
	pub props: HashMap<String, Property>,
}

impl Executable {
	pub fn run(self) -> Result<Value, Error> {
		let (tx, rx) = mpsc::channel();

		thread::spawn(move || {
			tx.send(self.render()).unwrap();
		});

		rx.recv().unwrap()
	}

	fn render(self) -> Result<Value, Error> {
		let main = self.main.clone();
		let state = self.create_state();
		let val = state.import(main)?;
		let wrapped = W(&val);
		let json = wrapped.try_into()?;

		Ok(json)
	}

	fn create_state(self) -> State {
		let state = State::default();

		let ctx = ContextInitializer::new(state.clone(), PathResolver::new_cwd_fallback());
		for (name, value) in self.props {
			let name = format!("{VARS_PREFIX}/{}", name.as_str());
			ctx.add_ext_var(name.into(), value.into());
		}

		state.set_context_initializer(ctx);
		state.set_import_resolver(FileImportResolver::new(vec![self.lib, self.vendor]));

		state
	}
}
