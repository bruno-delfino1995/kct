use crate::compiler::{
	property::{Name, Output, Property},
	Runtime,
};

use serde_json::Value;

pub struct Input(pub Value);

impl Property for Input {
	fn generate(&self, _: Runtime) -> Output {
		Output::Plain {
			name: Name::Input,
			value: self.0.clone(),
		}
	}
}
