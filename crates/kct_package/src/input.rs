use crate::compiler::{
	property::{Name, Output, Property},
	Compiler,
};

use serde_json::Value;

pub struct Input(pub Value);

impl Property for Input {
	fn name(&self) -> Name {
		Name::Input
	}

	fn generate(&self, _: &Compiler) -> Output {
		Output::Plain(self.0.clone())
	}
}
