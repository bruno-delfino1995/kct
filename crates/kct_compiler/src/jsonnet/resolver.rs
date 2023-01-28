use std::any::Any;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use jrsonnet_evaluator::error::Error as JrError;
use jrsonnet_evaluator::ImportResolver;
use jrsonnet_interner::IStr;

#[derive(Default)]
pub struct LibImportResolver {
	/// Library directories to search for file.
	/// Referred to as `jpath` in original jsonnet implementation.
	pub library_paths: Vec<PathBuf>,
}
impl ImportResolver for LibImportResolver {
	fn resolve_file(
		&self,
		from: &Path,
		path: &Path,
	) -> jrsonnet_evaluator::error::Result<Rc<Path>> {
		for library_path in self.library_paths.iter() {
			let mut cloned = library_path.clone();
			cloned.push(path);

			if cloned.exists() {
				return Ok(cloned.into());
			}
		}

		Err(JrError::ImportFileNotFound(from.to_path_buf(), path.to_path_buf()).into())
	}

	fn load_file_contents(&self, id: &Path) -> jrsonnet_evaluator::error::Result<IStr> {
		let mut file =
			File::open(id).map_err(|_e| JrError::ResolvedFileNotFound(id.to_path_buf()))?;
		let mut out = String::new();
		file.read_to_string(&mut out)
			.map_err(|_e| JrError::ImportBadFileUtf8(id.to_path_buf()))?;
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
		from: &Path,
		path: &Path,
	) -> jrsonnet_evaluator::error::Result<Rc<Path>> {
		let mut target = from.to_path_buf();
		target.push(path);

		let resolved = if target.exists() {
			Some(target.into())
		} else {
			from.parent()
				.map(|p| p.join(path))
				.and_then(|p| p.canonicalize().ok())
				.map(|p| p.into())
		};

		resolved.ok_or_else(|| {
			JrError::ImportFileNotFound(from.to_path_buf(), path.to_path_buf()).into()
		})
	}

	fn load_file_contents(&self, path: &Path) -> jrsonnet_evaluator::error::Result<IStr> {
		let mut file =
			File::open(path).map_err(|_e| JrError::ResolvedFileNotFound(path.to_path_buf()))?;
		let mut out = String::new();
		file.read_to_string(&mut out)
			.map_err(|_e| JrError::ImportBadFileUtf8(path.to_path_buf()))?;
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
		from: &Path,
		path: &Path,
	) -> jrsonnet_evaluator::error::Result<Rc<Path>> {
		for (i, resolver) in self.import_resolvers.iter().enumerate() {
			let resolved = resolver.resolve_file(from, path);

			if let Ok(ref path) = resolved {
				let path = {
					let base: PathBuf = path.to_path_buf();
					let mut base = base;

					base.push(format!("{i}.resolver"));

					base
				};

				return Ok(path.into());
			}
		}

		Err(JrError::ImportFileNotFound(from.to_path_buf(), path.to_path_buf()).into())
	}

	fn load_file_contents(&self, id: &Path) -> jrsonnet_evaluator::error::Result<IStr> {
		let error = JrError::ResolvedFileNotFound(id.to_path_buf()).into();

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

		resolver.load_file_contents(id.parent().unwrap())
	}

	unsafe fn as_any(&self) -> &dyn Any {
		panic!("this resolver can't be used as any")
	}
}
