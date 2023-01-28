mod prop;
mod resolver;

use self::resolver::*;

use crate::error::{Error, Result};

use std::collections::HashMap;
use std::path::PathBuf;

use jrsonnet_evaluator::error::{Error as JrError, LocError};
use jrsonnet_evaluator::trace::{ExplainingFormat, PathResolver};
use jrsonnet_evaluator::{EvaluationState, ManifestFormat, Val};
use serde_json::Value;

pub const VARS_PREFIX: &str = "kct.io";

pub struct Internal {
	pub vendor: PathBuf,
	pub lib: PathBuf,
	pub entrypoint: PathBuf,
	pub vars: HashMap<String, Val>,
}

impl Internal {
	pub fn compile(self) -> Result<Value> {
		let render_issue = |err: LocError| {
			let message = match err.error() {
				JrError::ImportSyntaxError { path, .. } => {
					format!("syntax error at {}", path.display())
				}
				err => err.to_string(),
			};

			Error::RenderIssue(message)
		};

		let state = self.create_state();
		for (name, value) in self.vars {
			let name = format!("{VARS_PREFIX}/{name}");
			state.add_ext_var(name.into(), value);
		}

		let parsed = state
			.evaluate_file_raw(&self.entrypoint)
			.map_err(render_issue)?;

		let rendered = state.manifest(parsed).map_err(render_issue)?.to_string();

		let json = serde_json::from_str(&rendered).map_err(|_err| Error::InvalidOutput)?;

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
