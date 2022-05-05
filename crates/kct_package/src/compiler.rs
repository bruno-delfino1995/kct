pub mod property;
mod resolvers;
pub mod runtime;
pub mod workspace;

use self::property::{Name, Output, Property};
use self::resolvers::*;
pub use self::runtime::Runtime;
pub use self::workspace::{Workspace, WorkspaceBuilder};

use crate::error::{Error, Result};

use jrsonnet_evaluator::{
	error::Error as JrError,
	error::LocError,
	trace::{ExplainingFormat, PathResolver},
	EvaluationState, ManifestFormat, Val,
};
use serde_json::Value;
use std::collections::HashMap;
use std::convert::From;
use std::rc::Rc;

pub const VARS_PREFIX: &str = "kct.io";

pub trait Validator: Fn(&Compiler) -> bool {}
impl<T: Fn(&Compiler) -> bool> Validator for T {}

#[derive(Clone, Default)]
pub struct Compiler {
	workspace: Workspace,
	properties: HashMap<Name, Rc<Output>>,
	validators: Vec<Rc<Box<dyn Validator>>>,
}

impl From<Workspace> for Compiler {
	fn from(workspace: Workspace) -> Self {
		Compiler {
			workspace,
			..Default::default()
		}
	}
}

impl Compiler {
	pub fn prop(mut self, prop: Box<dyn Property>) -> Self {
		let runtime: Runtime = (&self).into();

		let output = prop.generate(runtime);
		let name = output.name().clone();

		self.properties.insert(name, Rc::new(output));

		self
	}

	pub fn validator<F: 'static + Validator>(mut self, validator: F) -> Self {
		self.validators.push(Rc::new(Box::new(validator)));

		self
	}

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

		for validator in self.validators.iter() {
			if !validator(&self) {
				return Err(Error::InvalidInput);
			}
		}

		let state = self.create_state();

		let variables = self.create_ext_vars();
		for (name, value) in variables {
			let name = format!("{}/{}", VARS_PREFIX, name);
			state.add_ext_var(name.into(), value);
		}

		let parsed = state
			.evaluate_file_raw(&self.workspace.entrypoint().to_path_buf())
			.map_err(render_issue)?;

		let rendered = state.manifest(parsed).map_err(render_issue)?.to_string();

		let json = serde_json::from_str(&rendered).map_err(|_err| Error::InvalidOutput)?;

		Ok(json)
	}

	fn create_ext_vars(&self) -> HashMap<String, Val> {
		let from_prop = |p: property::Name| -> (String, Val) {
			let default = Val::Null;
			let name = p.as_str();
			let property = self.properties.get(&p);

			let val = property
				.map(|value| {
					let copy = (**value).clone();

					copy.into()
				})
				.unwrap_or(default);

			(String::from(name), val)
		};

		vec![
			from_prop(Name::Package),
			from_prop(Name::Release),
			from_prop(Name::Input),
			from_prop(Name::Include),
			from_prop(Name::File),
		]
		.into_iter()
		.collect()
	}

	fn create_state(&self) -> EvaluationState {
		let state = EvaluationState::default();
		let resolver = PathResolver::Absolute;
		state.set_trace_format(Box::new(ExplainingFormat { resolver }));

		state.with_stdlib();

		let vendor = self.workspace.vendor().to_path_buf();
		let lib = self.workspace.lib().to_path_buf();

		let relative_resolver = Box::new(RelativeImportResolver);

		let lib_resolver = Box::new(LibImportResolver {
			library_paths: vec![lib, vendor],
		});

		let resolver = AggregatedImportResolver::default()
			.push(relative_resolver)
			.push(lib_resolver);

		state.set_import_resolver(Box::new(resolver));

		state.set_manifest_format(ManifestFormat::Json(0));

		state
	}
}
