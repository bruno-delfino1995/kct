use jrsonnet_evaluator::error::Error as JrError;
use jrsonnet_evaluator::ImportResolver;
use jrsonnet_interner::IStr;
use std::any::Any;
use std::borrow::Borrow;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use std::rc::Rc;

#[derive(Default)]
pub struct LibImportResolver {
	/// Library directories to search for file.
	/// Referred to as `jpath` in original jsonnet implementation.
	pub library_paths: Vec<PathBuf>,
}
impl ImportResolver for LibImportResolver {
	fn resolve_file(
		&self,
		from: &PathBuf,
		path: &PathBuf,
	) -> jrsonnet_evaluator::error::Result<Rc<PathBuf>> {
		for library_path in self.library_paths.iter() {
			let mut cloned = library_path.clone();
			cloned.push(path);
			if cloned.exists() {
				return Ok(Rc::new(cloned));
			}
		}

		Err(JrError::ImportFileNotFound(from.clone(), path.clone()).into())
	}

	fn load_file_contents(&self, id: &PathBuf) -> jrsonnet_evaluator::error::Result<IStr> {
		let mut file = File::open(id).map_err(|_e| JrError::ResolvedFileNotFound(id.clone()))?;
		let mut out = String::new();
		file.read_to_string(&mut out)
			.map_err(|_e| JrError::ImportBadFileUtf8(id.clone()))?;
		Ok(out.into())
	}
	unsafe fn as_any(&self) -> &dyn Any {
		panic!("this resolver can't be used as any")
	}
}

pub struct RelativeImportResolver;

impl ImportResolver for RelativeImportResolver {
	fn resolve_file(
		&self,
		from: &PathBuf,
		path: &PathBuf,
	) -> jrsonnet_evaluator::error::Result<Rc<PathBuf>> {
		let mut target = from.clone();
		target.push(path);

		let resolved = if target.exists() {
			Some(Rc::new(target))
		} else {
			from.parent()
				.map(|p| p.join(path))
				.and_then(|p| p.canonicalize().ok())
				.map(Rc::new)
		};

		resolved.ok_or_else(|| JrError::ImportFileNotFound(from.clone(), path.clone()).into())
	}

	fn load_file_contents(&self, path: &PathBuf) -> jrsonnet_evaluator::error::Result<IStr> {
		let mut file =
			File::open(path).map_err(|_e| JrError::ResolvedFileNotFound(path.clone()))?;
		let mut out = String::new();
		file.read_to_string(&mut out)
			.map_err(|_e| JrError::ImportBadFileUtf8(path.clone()))?;
		Ok(out.into())
	}

	unsafe fn as_any(&self) -> &dyn Any {
		panic!("this resolver can't be used as any")
	}
}

#[derive(Default)]
pub struct AggregatedImportResolver {
	import_resolvers: Vec<Box<dyn ImportResolver>>,
}

impl AggregatedImportResolver {
	pub fn push(mut self, resolver: Box<dyn ImportResolver>) -> Self {
		self.import_resolvers.push(resolver);

		self
	}
}

impl ImportResolver for AggregatedImportResolver {
	fn resolve_file(
		&self,
		from: &PathBuf,
		path: &PathBuf,
	) -> jrsonnet_evaluator::error::Result<Rc<PathBuf>> {
		for (i, resolver) in self.import_resolvers.iter().enumerate() {
			let resolved = resolver.resolve_file(from, path);

			if let Ok(ref path) = resolved {
				let path = {
					let base: &PathBuf = path.borrow();
					let mut base = base.clone();

					base.push(format!("{}.resolver", i));

					base
				};

				return Ok(Rc::new(path));
			}
		}

		println!("Failed because every resolver failed");

		Err(JrError::ImportFileNotFound(from.clone(), path.clone()).into())
	}

	fn load_file_contents(&self, id: &PathBuf) -> jrsonnet_evaluator::error::Result<IStr> {
		let error = JrError::ResolvedFileNotFound(id.clone()).into();

		let is_from_resolver = id.extension().map_or(false, |ext| ext.eq("resolver"));
		if !is_from_resolver {
			return Err(error);
		}

		let resolver = id
			.file_stem()
			.and_then(|stem| stem.to_str())
			.and_then(|stem| stem.parse::<usize>().ok())
			.and_then(|index| self.import_resolvers.get(index))
			.ok_or(error)?;

		resolver.load_file_contents(&id.parent().unwrap().to_path_buf())
	}

	unsafe fn as_any(&self) -> &dyn Any {
		panic!("this resolver can't be used as any")
	}
}
