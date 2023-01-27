mod context;
mod error;
mod prop;
mod resolver;
mod runtime;
mod target;

pub mod extension;

use self::error::Result;
use self::extension::{Extension, Name, Plugins};
use self::resolver::*;

pub use self::context::{Context, ContextBuilder};
pub use self::error::Error;
pub use self::runtime::Runtime;
pub use self::target::{Target, TargetBuilder};

use std::collections::HashMap;
use std::convert::From;
use std::rc::Rc;

use jrsonnet_evaluator::error::{Error as JrError, LocError};
use jrsonnet_evaluator::trace::{ExplainingFormat, PathResolver};
use jrsonnet_evaluator::{EvaluationState, ManifestFormat, Val};
use serde_json::Value;

pub const VARS_PREFIX: &str = "kct.io";

pub trait Validator: Fn(&Compiler) -> bool {}
impl<T: Fn(&Compiler) -> bool> Validator for T {}

#[derive(Clone, Debug)]
pub struct Release {
	pub name: String,
}

pub struct Input(pub Value);

pub struct Compiler {
	context: Context,
	workspace: Target,
	plugins: Plugins,
	validators: Vec<Rc<Box<dyn Validator>>>,
}

impl Compiler {
	pub fn new(ctx: &Context, wk: Target) -> Self {
		let mut res = Self {
			context: ctx.clone(),
			workspace: wk,
			plugins: Plugins::new(),
			validators: vec![],
		};

		res = match ctx.release() {
			Some(release) => res.extend(Box::new(release.clone())),
			None => res,
		};

		res
	}

	pub fn extend(mut self, ext: Box<dyn Extension>) -> Self {
		let runtime: Runtime = (&self).into();

		self.plugins.put(ext.plug(runtime));

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
			let name = format!("{VARS_PREFIX}/{name}");
			state.add_ext_var(name.into(), value);
		}

		let parsed = state
			.evaluate_file_raw(self.workspace.main())
			.map_err(render_issue)?;

		let rendered = state.manifest(parsed).map_err(render_issue)?.to_string();

		let json = serde_json::from_str(&rendered).map_err(|_err| Error::InvalidOutput)?;

		Ok(json)
	}

	fn create_ext_vars(&self) -> HashMap<String, Val> {
		let from_plugin = |p: Name| -> (String, Val) {
			let default = Val::Null;
			let name = p.as_str();
			let property = self.plugins.get(p);

			let val = property
				.map(|value| {
					let copy = (*value).clone();

					copy.into()
				})
				.unwrap_or(default);

			(String::from(name), val)
		};

		vec![
			from_plugin(Name::Package),
			from_plugin(Name::Release),
			from_plugin(Name::Input),
			from_plugin(Name::Include),
			from_plugin(Name::File),
		]
		.into_iter()
		.collect()
	}

	fn create_state(&self) -> EvaluationState {
		let state = EvaluationState::default();
		let resolver = PathResolver::Absolute;
		state.set_trace_format(Box::new(ExplainingFormat { resolver }));

		state.with_stdlib();

		let vendor = self.context.vendor().to_path_buf();
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
