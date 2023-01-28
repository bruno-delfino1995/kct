use crate::extension::{Extension, Name, Plugin, Property};
use crate::{Input, Runtime};

impl Extension for Input {
	fn plug(&self, _: Runtime) -> Plugin {
		Plugin::Create(Property::Primitive(Name::Input, self.0.clone()))
	}
}
