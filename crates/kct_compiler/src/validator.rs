use serde_json::Value;

pub trait Predicate: Fn(&Value) -> Result<(), String> {}
impl<T: Fn(&Value) -> Result<(), String>> Predicate for T {}

pub struct Validator {
	predicate: Box<dyn Predicate>,
}

impl Validator {
	pub fn new(predicate: Box<dyn Predicate>) -> Self {
		Self { predicate }
	}

	pub fn run(&self, value: &Value) -> Result<(), String> {
		(self.predicate)(value)
	}
}
