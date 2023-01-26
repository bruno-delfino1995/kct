use crate::compiler::{
	property::{Callback, Finalize, Function, Gc, Name, Output, Property, Trace},
	Runtime,
};

use jrsonnet_gc::unsafe_empty_trace;

use globwalk::{DirEntry, GlobWalkerBuilder};
use serde_json::{Map, Value};
use std::path::{Path, PathBuf};
use std::{collections::HashMap, fs};
use tera::{Context, Tera};

const TEMPLATES_FOLDER: &str = "files";

pub struct File;

struct Val(Value);

impl Finalize for Val {}
unsafe impl Trace for Val {
	unsafe_empty_trace!();
}

#[derive(Trace, Finalize)]
struct Handler {
	root: PathBuf,
	input: Gc<Val>,
}

impl Callback for Handler {
	fn call(&self, params: HashMap<String, Value>) -> Result<Value, String> {
		let name = match params.get("name") {
			None => return Err("name is required".into()),
			Some(name) => name,
		};

		let file = match name {
			Value::String(name) => name,
			_ => return Err("name should be a string".into()),
		};

		let compiled = compile_template(&self.root, file, &self.input.0)?;

		if compiled.is_empty() {
			Err(format!("No template found for glob {}", file))
		} else if compiled.len() == 1 {
			Ok(Value::String(compiled.into_iter().next().unwrap()))
		} else {
			Ok(Value::Array(
				compiled.into_iter().map(Value::String).collect(),
			))
		}
	}
}

impl Property for File {
	fn generate(&self, runtime: Runtime) -> Output {
		let root = runtime.workspace.dir().to_path_buf();

		let input = runtime
			.properties
			.get(&Name::Input)
			.and_then(|v| match v.as_ref() {
				Output::Plain { value, .. } => Some(value),
				_ => None,
			})
			.unwrap_or(&Value::Null)
			.clone();

		let params = vec![String::from("name")];
		let handler = Handler {
			root,
			input: Gc::new(Val(input)),
		};
		let function = Function {
			params,
			handler: Gc::new(Box::new(handler)),
		};

		let name = Name::File;
		Output::Callback { name, function }
	}
}

fn compile_template(
	root: &Path,
	glob: &str,
	input: &Value,
) -> std::result::Result<Vec<String>, String> {
	let mut templates_dir = root.to_path_buf();
	templates_dir.push(TEMPLATES_FOLDER);

	if !templates_dir.exists() {
		return Err(String::from("No files folder to search for templates"));
	}

	let globwalker = GlobWalkerBuilder::new(templates_dir, glob)
		.build()
		.map_err(|err| format!("Invalid glob provided ({}): {}", glob, err))?;

	let entries: Vec<DirEntry> = globwalker
		.collect::<std::result::Result<_, _>>()
		.map_err(|err| format!("Unable to resolve globs: {}", err))?;

	let mut paths: Vec<PathBuf> = entries.into_iter().map(DirEntry::into_path).collect();

	paths.sort();

	let contents: Vec<String> = paths
		.into_iter()
		.map(fs::read_to_string)
		.collect::<std::result::Result<_, _>>()
		.map_err(|err| format!("Unable to read templates: {}", err))?;

	let context = match input {
		Value::Null => Context::from_serialize(Value::Object(Map::new())).unwrap(),
		_ => Context::from_serialize(input).unwrap(),
	};

	let compiled: Vec<String> = contents
		.into_iter()
		.map(|content| Tera::one_off(&content, &context, true))
		.collect::<std::result::Result<_, _>>()
		.map_err(|err| format!("Unable to compile templates: {}", err))?;

	Ok(compiled)
}
