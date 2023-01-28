mod context;
mod error;
mod jsonnet;
mod target;
mod validator;

pub mod property;

use self::error::Result;
use self::jsonnet::Executable;
use self::property::{Name, Prop, Property};

pub use self::context::Context;
pub use self::error::Error;
pub use self::target::{Target, TargetBuilder};
pub use self::validator::Validator;

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use context::ContextBuilder;
use jrsonnet_evaluator::Val;
use serde_json::Value;

#[derive(Clone, Debug)]
pub struct Release {
	pub name: String,
}

pub struct Input(pub Value);

pub(crate) struct System {
	context: Context,
	target: Target,
	props: HashMap<Name, Prop>,
	checks: Vec<Validator>,
}

impl System {
	fn generate(mut self) -> Result<Executable> {
		let input = match self.props.get(&Name::Input) {
			Some(Prop::Primitive(_, v)) => Some(v),
			_ => None,
		};

		self.validate(input)?;

		Ok(Executable {
			vendor: self.context.vendor().to_path_buf(),
			lib: self.target.lib().to_path_buf(),
			entrypoint: self.target.main().to_path_buf(),
			vars: self.properties(),
		})
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

	fn validate(&self, input: Option<&Value>) -> Result<()> {
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

#[derive(Clone)]
pub struct Runtime {
	context: Context,
	target: Target,
}

impl Runtime {
	pub fn context(&self) -> &Context {
		&self.context
	}

	pub fn target(&self) -> &Target {
		&self.target
	}
}

pub struct Compiler {
	context: ContextBuilder,
	target: Option<Target>,
	dynamics: HashMap<Name, Box<dyn Property>>,
	statics: HashMap<Name, Prop>,
	checks: Vec<Validator>,
}

impl Compiler {
	pub fn bootstrap(root: &Path) -> Self {
		let context = ContextBuilder::default().root(root.to_path_buf());

		Self {
			context,
			target: None,
			dynamics: HashMap::new(),
			statics: HashMap::new(),
			checks: vec![],
		}
	}

	pub fn inherit(ctx: &Context) -> Self {
		let context = ContextBuilder::wrap(ctx.clone());

		Self {
			context,
			target: None,
			dynamics: HashMap::new(),
			statics: HashMap::new(),
			checks: vec![],
		}
	}

	pub fn with_dynamic_prop(mut self, prop: Option<Box<dyn Property>>) -> Self {
		if let Some(prop) = prop {
			self.dynamics.insert(prop.name(), prop);
		}

		self
	}

	pub fn with_static_prop(mut self, prop: Option<Prop>) -> Self {
		if let Some(prop) = prop {
			self.statics.insert(*prop.name(), prop);
		}

		self
	}

	pub fn with_check(mut self, check: Validator) -> Self {
		self.checks.push(check);

		self
	}

	pub fn with_target(mut self, target: Target) -> Self {
		match self.target {
			Some(_) => self,
			None => {
				self.target = Some(target);

				self
			}
		}
	}

	pub fn with_release(mut self, release: Option<Release>) -> Self {
		self.context = self.context.release(release.clone());

		let prop = release.map(|r| (&r).into());
		self.with_static_prop(prop)
	}

	pub fn with_vendor(mut self, vendor: PathBuf) -> Self {
		self.context = self.context.vendor(vendor);

		self
	}

	pub fn compile(self) -> Result<Value> {
		let system: System = self.try_into()?;
		let executable = system.generate()?;

		executable.run()
	}
}

impl TryInto<System> for Compiler {
	type Error = Error;

	fn try_into(mut self) -> std::result::Result<System, Self::Error> {
		let target = self.target.ok_or(Error::NoTarget)?;

		let context = self.context.build().map_err(Error::Wrapped)?;

		let runtime = Runtime { context, target };
		let dynamics: HashMap<Name, Prop> = self
			.dynamics
			.into_iter()
			.map(|(n, p)| (n, p.generate(&runtime)))
			.collect();

		let props = {
			let mut statics = std::mem::take(&mut self.statics);

			statics.extend(dynamics);

			statics
		};

		Ok(System {
			props,
			checks: self.checks,
			context: runtime.context,
			target: runtime.target,
		})
	}
}
