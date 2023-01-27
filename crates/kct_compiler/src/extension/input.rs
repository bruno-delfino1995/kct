use crate::{
	extension::{Extension, Name, Plugin},
	{Input, Runtime},
};

impl Extension for Input {
	fn plug(&self, _: Runtime) -> Plugin {
		Plugin::Property {
			name: Name::Input,
			value: self.0.clone(),
		}
	}
}
