use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use globwalk::{DirEntry, GlobWalkerBuilder};
use kct_compiler::extension::{Callback, Extension, Function, Name, Plugin, Property};
use kct_compiler::Runtime;
use serde_json::{Map, Value};
use tera::{Context, Tera};

const TEMPLATES_FOLDER: &str = "files";

pub struct File;

struct Handler {
	root: PathBuf,
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

		let input = params.get("input").cloned().unwrap_or(Value::Null);

		let compiled = compile_template(&self.root, file, &input)?;

		if compiled.is_empty() {
			Err(format!("No template found for glob {file}"))
		} else if compiled.len() == 1 {
			Ok(Value::String(compiled.into_iter().next().unwrap()))
		} else {
			Ok(Value::Array(
				compiled.into_iter().map(Value::String).collect(),
			))
		}
	}
}

impl Extension for File {
	fn plug(&self, runtime: Runtime) -> Plugin {
		let root = runtime.target.dir().to_path_buf();

		let params = vec![String::from("name"), String::from("input")];
		let handler = Handler { root };
		let function = Function {
			params,
			handler: Rc::new(handler),
		};

		let name = Name::File;
		Plugin::Create(Property::Callable(name, function))
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
		.map_err(|err| format!("Invalid glob provided ({glob}): {err}"))?;

	let entries: Vec<DirEntry> = globwalker
		.collect::<std::result::Result<_, _>>()
		.map_err(|err| format!("Unable to resolve globs: {err}"))?;

	let mut paths: Vec<PathBuf> = entries.into_iter().map(DirEntry::into_path).collect();

	paths.sort();

	let contents: Vec<String> = paths
		.into_iter()
		.map(fs::read_to_string)
		.collect::<std::result::Result<_, _>>()
		.map_err(|err| format!("Unable to read templates: {err}"))?;

	let context = match input {
		Value::Null => Context::from_serialize(Value::Object(Map::new())).unwrap(),
		_ => Context::from_serialize(input).unwrap(),
	};

	let compiled: Vec<String> = contents
		.into_iter()
		.map(|content| Tera::one_off(&content, &context, true))
		.collect::<std::result::Result<_, _>>()
		.map_err(|err| format!("Unable to compile templates: {err}"))?;

	Ok(compiled)
}
