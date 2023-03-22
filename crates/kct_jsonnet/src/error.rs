use std::fmt;

use jrsonnet_evaluator::error::{Error as LocError, StackTrace, StackTraceElement};
use serde_json::Error as JsonError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
	#[error("Render failed due to \"{0}\"")]
	Render(String, #[source] Trace),
	#[error("Template couldn't be parsed as JSON")]
	InvalidOutput(#[from] JsonError),
}

impl From<LocError> for Error {
	fn from(err: LocError) -> Self {
		let error = err.error();
		let trace = err.trace();

		Error::Render(error.to_string(), Trace::new(trace))
	}
}

#[derive(Error, Debug)]
pub struct Trace(Vec<String>);

impl Trace {
	fn new(trace: &StackTrace) -> Self {
		let traces = trace
			.0
			.iter()
			.filter(|el| el.location.is_some())
			.map(Trace::format_element)
			.collect();

		Self(traces)
	}

	fn format_element(el: &StackTraceElement) -> String {
		let desc = &el.desc;
		let location = {
			let loc = el.location.as_ref().unwrap().clone();
			let path_with_resolver = loc.0.source_path().path().unwrap();
			let begin = loc.1;
			let end = loc.2;

			let file = path_with_resolver.parent().unwrap().to_path_buf();

			format!("{}:{begin}-{end}", file.display())
		};

		format!("{desc}\n\tat {location}\n")
	}
}

impl fmt::Display for Trace {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		self.0.iter().try_for_each(|err| writeln!(f, "{err}"))
	}
}
