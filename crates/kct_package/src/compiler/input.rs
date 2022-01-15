use crate::property::{Name, Property};
use serde_json::Value;

pub struct Input(pub Value);

impl Property for Input {
	fn name(&self) -> Name {
		Name::Input
	}

	fn generate(&self) -> Value {
		self.0.clone()
	}
}
