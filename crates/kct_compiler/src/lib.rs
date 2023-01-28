mod context;
mod error;
mod internal;
mod runtime;
mod target;
mod validator;

pub mod property;

use self::error::Result;
use self::internal::Internal;
use self::property::{Name, Prop, Property};

pub use self::context::{Context, ContextBuilder};
pub use self::error::Error;
pub use self::runtime::Runtime;
pub use self::target::{Target, TargetBuilder};
pub use self::validator::Validator;

use std::collections::HashMap;

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
	props: HashMap<Name, Prop>,
	checks: Vec<Validator>,
}

impl Compiler {
	pub fn new(context: &Context, target: &Target) -> Self {
		let mut res = Self {
			context: context.clone(),
			target: target.clone(),
			props: HashMap::new(),
			checks: Vec::new(),
		};

		res = match context.release() {
			Some(release) => res.inject(Box::new(release.clone())),
			None => res,
		};

		res
	}

	pub fn inject(mut self, property: Box<dyn Property>) -> Self {
		let runtime: Runtime = (&self).into();
		let prop = property.generate(runtime);

		self.props.insert(*prop.name(), prop);

		self
	}

	pub fn ensure(mut self, check: Validator) -> Self {
		self.checks.push(check);

		self
	}

	pub fn compile(mut self, input: Option<Value>) -> Result<Value> {
		self.validate(&input)?;

		self = match input {
			Some(input) => self.inject(Box::new(Input(input))),
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

	fn properties(&mut self) -> HashMap<String, Val> {
		let props = std::mem::take(&mut self.props);

		let mut defaults: HashMap<String, Val> = Name::all()
			.into_iter()
			.map(|n| (n.as_str().to_string(), Val::Null))
			.collect();

		let configured: HashMap<String, Val> = props
			.into_iter()
			.map(|(n, p)| {
				let name = n.as_str();

				(name.to_string(), p.into())
			})
			.collect();

		defaults.extend(configured);

		defaults
	}

	fn validate(&self, input: &Option<Value>) -> Result<()> {
		let is_empty = self.checks.is_empty();

		let input = match (is_empty, input.as_ref()) {
			(true, None) => return Ok(()),
			(true, Some(_)) => return Err(Error::NoValidator),
			(false, None) => return Err(Error::NoInput),
			(false, Some(input)) => input,
		};

		for check in &self.checks {
			check.run(input).map_err(Error::InvalidInput)?;
		}

		Ok(())
	}
}
