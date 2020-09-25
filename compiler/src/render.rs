pub mod error;

use self::error::{Error, Result};
use crate::package::Package;
use jrsonnet_evaluator::{
	error::LocError,
	trace::{ExplainingFormat, PathResolver},
	EvaluationState, FileImportResolver, Val,
};
use serde_json::Value;

const VALUES_PARAM: &str = "values";

pub fn render(pkg: &Package, values: Option<Value>) -> Result<Value> {
	let values = validate_values(pkg, values)?;
	let state = create_state(pkg);

	let render_issue = |err: LocError| Error::RenderIssue(format!("{}", err.error()));

	state
		.add_tla_code(VALUES_PARAM.into(), values.to_string().into())
		.map_err(render_issue)?;

	let parsed = state
		.evaluate_file_raw_nocwd(&pkg.spec.main)
		.map_err(render_issue)?;

	let parsed = match parsed {
		Val::Func(_) => parsed,
		_ => return Err(Error::RenderIssue(String::from("Template is not a TLA"))),
	};

	let wrapped = state.with_tla(parsed).map_err(render_issue)?;

	let rendered = state.manifest(wrapped).map_err(render_issue)?.to_string();

	let json = serde_json::from_str(&rendered).map_err(|_err| Error::InvalidOutput)?;

	Ok(json)
}

fn create_state(pkg: &Package) -> EvaluationState {
	let state = EvaluationState::default();
	let resolver = PathResolver::Absolute;
	state.set_trace_format(Box::new(ExplainingFormat { resolver }));

	state.with_stdlib();

	state.set_import_resolver(Box::new(FileImportResolver {
		library_paths: vec![pkg.root.clone()],
	}));

	state
}

fn validate_values(pkg: &Package, values: Option<Value>) -> Result<Value> {
	let (schema, values) = match (&pkg.schema, values) {
		(None, None) => return Ok(Value::Null),
		(None, Some(_)) => return Err(Error::NoSchema),
		(Some(_), None) => return Err(Error::NoValues),
		(Some(schema), Some(value)) => (schema, value),
	};

	if schema.validate(&values) {
		Ok(values)
	} else {
		Err(Error::InvalidValues)
	}
}
