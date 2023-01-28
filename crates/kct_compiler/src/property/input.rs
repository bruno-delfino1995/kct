use crate::property::{Name, Prop, Property};
use crate::{Input, Runtime};

impl Property for Input {
	fn generate(&self, _: Runtime) -> Prop {
		Prop::Primitive(Name::Input, self.0.clone())
	}
}
