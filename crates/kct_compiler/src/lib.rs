mod context;
mod error;
mod target;
mod validator;

pub mod property;

use self::property::{Generator, Property};
use self::property::{Name, Prop};

pub use self::context::Context;
pub use self::error::Error;
pub use self::target::{Target, TargetBuilder};
pub use self::validator::Validator;

use std::collections::HashMap;

use anyhow::Result;
use kct_jsonnet::Executable;
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
	fn generate(mut self) -> Result<Executable, Error> {
		let input = match self.props.get(&Name::Input) {
			Some(prop) => prop.value(),
			_ => None,
		};

		self.validate(input)?;

		Ok(Executable {
			vendor: self.context.vendor().to_path_buf(),
			lib: self.target.lib().to_path_buf(),
			main: self.target.main().to_path_buf(),
			props: self.properties(),
		})
	}

	fn properties(&mut self) -> HashMap<String, Property> {
		let props = std::mem::take(&mut self.props);

		let mut defaults: HashMap<Name, Prop> = Name::all()
			.into_iter()
			.map(|n| (n, Prop::primitive(n, Value::Null)))
			.collect();

		defaults.extend(props);

		defaults
			.into_iter()
			.map(|(k, v)| {
				let (_, value) = v.take();
				let name = k.as_str().to_string();

				(name, value)
			})
			.collect()
	}

	fn validate(&self, input: Option<&Value>) -> Result<(), Error> {
		let is_empty = self.checks.is_empty();

		let input = match (is_empty, input.as_ref()) {
			(true, None) => return Ok(()),
			(true, Some(Value::Null)) => return Ok(()),
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
	context: Context,
	target: Option<Target>,
	dynamics: HashMap<Name, Box<dyn Generator>>,
	statics: HashMap<Name, Prop>,
	checks: Vec<Validator>,
}

impl Compiler {
	pub fn new(context: &Context) -> Self {
		Self {
			context: context.clone(),
			target: None,
			dynamics: HashMap::new(),
			statics: HashMap::new(),
			checks: vec![],
		}
	}

	pub fn with_dynamic_prop(mut self, prop: Option<Box<dyn Generator>>) -> Self {
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

	pub fn compile(self) -> Result<Value, Error> {
		let system: System = self.try_into()?;
		let executable = system.generate()?;
		let value = executable.run()?;

		Ok(value)
	}
}

impl TryInto<System> for Compiler {
	type Error = Error;

	fn try_into(mut self) -> Result<System, Self::Error> {
		let target = self.target.ok_or(Error::NoTarget)?;

		let context = self.context;
		let release = context.release().clone().map(|r| (&r).into());

		let runtime = Runtime { context, target };
		let dynamics: HashMap<Name, Prop> = self
			.dynamics
			.into_iter()
			.map(|(n, p)| (n, p.generate(&runtime)))
			.collect();

		let props = {
			let mut base = std::mem::take(&mut self.statics);

			base.extend(dynamics);

			if let Some(release) = release {
				base.insert(Name::Release, release);
			}

			base
		};

		Ok(System {
			props,
			checks: self.checks,
			context: runtime.context,
			target: runtime.target,
		})
	}
}
