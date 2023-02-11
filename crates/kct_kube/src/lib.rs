mod client;
mod ingestor;

pub mod error;

use self::client::Client;
use self::ingestor::Ingestor;

pub use crate::error::Root as Error;

use std::path::PathBuf;

use anyhow::Result;
use serde_json::Value;

#[derive(Debug)]
pub struct Manifest(PathBuf, Value);

impl From<Manifest> for (PathBuf, Value) {
	fn from(val: Manifest) -> Self {
		let path = val.0;
		let manifest = val.1;

		(path, manifest)
	}
}

impl From<Manifest> for (PathBuf, String) {
	fn from(val: Manifest) -> Self {
		let path = val.0;
		let manifest = serde_yaml::to_string(&val.1).unwrap();

		(path, manifest)
	}
}

pub struct Kube {
	value: Value,
	ingestor: Ingestor,
}

impl Kube {
	pub fn builder() -> Builder {
		Default::default()
	}

	pub fn render(&self) -> Result<Vec<Manifest>, Error> {
		self.ingestor.ingest(&self.value)
	}

	pub async fn install(self) -> Result<()> {
		let mut client = Client::try_new().await?;
		let manifests = self
			.render()?
			.into_iter()
			.map(|Manifest(_, val)| val)
			.collect();

		client.apply(manifests).await
	}

	pub async fn uninstall(self) -> Result<()> {
		let mut client = Client::try_new().await?;
		let manifests = self
			.render()?
			.into_iter()
			.map(|Manifest(_, val)| val)
			.collect();

		client.delete(manifests).await
	}
}

impl TryFrom<Kube> for Vec<Manifest> {
	type Error = Error;

	fn try_from(source: Kube) -> std::result::Result<Self, Self::Error> {
		source.render()
	}
}

#[derive(Default)]
pub struct Builder {
	value: Option<Value>,
	only: Vec<PathBuf>,
	except: Vec<PathBuf>,
}

impl Builder {
	pub fn value(mut self, value: Value) -> Self {
		match self.value {
			Some(_) => self,
			None => {
				self.value = Some(value);

				self
			}
		}
	}

	pub fn only(mut self, only: Vec<PathBuf>) -> Self {
		self.only = only;

		self
	}

	pub fn except(mut self, except: Vec<PathBuf>) -> Self {
		self.except = except;

		self
	}

	pub fn build(self) -> Result<Kube, Error> {
		let value = self.value.ok_or(Error::MissingValue)?;
		let ingestor = Ingestor::new(self.only, self.except);

		Ok(Kube { ingestor, value })
	}
}
