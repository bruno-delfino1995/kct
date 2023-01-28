mod context;
mod error;
mod internal;
mod runtime;
mod target;

pub mod extension;

use self::error::Result;
use self::extension::{Extension, Name, Plugins};
use self::internal::Internal;

pub use self::context::{Context, ContextBuilder};
pub use self::error::Error;
pub use self::runtime::Runtime;
pub use self::target::{Target, TargetBuilder};

use std::collections::HashMap;
use std::rc::Rc;

use extension::Predicate;
use jrsonnet_evaluator::Val;
use serde_json::Value;

#[derive(Clone, Debug)]
pub struct Release {
	pub name: String,
}

pub struct Input(pub Value);

pub struct Compiler {
	context: Context,
	target: Target,
	plugins: Plugins,
}

impl Compiler {
	pub fn new(context: &Context, target: &Target) -> Self {
		let mut res = Self {
			context: context.clone(),
			target: target.clone(),
			plugins: Plugins::new(),
		};

		res = match context.release() {
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

	pub fn compile(mut self, input: Option<Value>) -> Result<Value> {
		self.validate(&input)?;

		self = match input {
			Some(input) => self.extend(Box::new(Input(input))),
			None => self,
		};

		let internal = Internal {
			vendor: self.context.vendor().to_path_buf(),
			lib: self.target.lib().to_path_buf(),
			entrypoint: self.target.main().to_path_buf(),
			vars: self.properties(),
		};

		internal.compile()
	}

	fn properties(&self) -> HashMap<String, Val> {
		let mut defaults: HashMap<String, Val> = Name::all()
			.into_iter()
			.map(|n| (n.as_str().to_string(), Val::Null))
			.collect();

		let configured: HashMap<String, Val> = self
			.plugins
			.iter()
			.filter_map(|p| p.property())
			.map(|p| {
				let name = p.name().as_str();

				(name.to_string(), (*p).clone().into())
			})
			.collect();

		defaults.extend(configured);

		defaults
	}

	fn validate(&self, input: &Option<Value>) -> Result<()> {
		let funcs: Vec<Rc<dyn Predicate>> =
			self.plugins.iter().filter_map(|p| p.validator()).collect();

		let is_empty = funcs.is_empty();

		let input = match (is_empty, input.as_ref()) {
			(true, None) => return Ok(()),
			(true, Some(_)) => return Err(Error::NoValidator),
			(false, None) => return Err(Error::NoInput),
			(false, Some(input)) => input,
		};

		for func in funcs {
			func(input).map_err(Error::InvalidInput)?;
		}

		Ok(())
	}
}
