use crate::property::{Name, Prop};
use crate::Input;

impl From<&Input> for Prop {
	fn from(val: &Input) -> Self {
		Prop::Primitive(Name::Input, val.0.clone())
	}
}
