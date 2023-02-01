mod error;
mod resolver;

pub mod property;

use crate::property::Property;
use crate::resolver::*;

pub use crate::error::Error;

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::mpsc;
use std::thread;

use anyhow::Result;
use jrsonnet_evaluator::trace::{ExplainingFormat, PathResolver};
use jrsonnet_evaluator::{EvaluationState, ManifestFormat};
use serde_json::Value;

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
		let state = self.create_state();
		for (name, value) in self.props {
			let name = format!("{VARS_PREFIX}/{}", name.as_str());
			state.add_ext_var(name.into(), value.into());
		}

		let parsed = state.evaluate_file_raw(&self.main).map_err(Error::from)?;

		let rendered = state.manifest(parsed).map_err(Error::from)?.to_string();

		let json = serde_json::from_str(&rendered)?;

		Ok(json)
	}

	fn create_state(&self) -> EvaluationState {
		let state = EvaluationState::default();
		let resolver = PathResolver::Absolute;
		state.set_trace_format(Box::new(ExplainingFormat { resolver }));

		state.with_stdlib();

		let relative_resolver = Box::new(RelativeImportResolver);

		let lib_resolver = Box::new(LibImportResolver {
			library_paths: vec![self.lib.clone(), self.vendor.clone()],
		});

		let resolver = AggregatedImportResolver::default()
			.push(relative_resolver)
			.push(lib_resolver);

		state.set_import_resolver(Box::new(resolver));

		state.set_manifest_format(ManifestFormat::Json(0));

		state
	}
}
