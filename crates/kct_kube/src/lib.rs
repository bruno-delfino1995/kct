mod ingest;

pub mod error;

use crate::ingest::Filter;

pub use crate::error::Root as Error;

use std::path::PathBuf;

use anyhow::{bail, Result};

use kube::api::{Api, DynamicObject, Patch, PatchParams, ResourceExt};
use kube::core::GroupVersionKind;
use kube::discovery::{ApiCapabilities, ApiResource, Discovery, Scope};
use kube::Client;
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
	manifests: Vec<Manifest>,
}

impl Kube {
	pub fn builder() -> Builder {
		Default::default()
	}

	pub async fn apply(self) -> Result<()> {
		let manifests = self.manifests;

		let client = Client::try_default().await?;
		let discovery = Discovery::new(client.clone()).run().await?;
		let ssapply = PatchParams::apply("kubectl-light").force();
		for Manifest(_, doc) in manifests {
			let obj: DynamicObject = serde_json::from_value(doc)?;
			let gvk = if let Some(tm) = &obj.types {
				GroupVersionKind::try_from(tm)?
			} else {
				bail!("cannot apply object without valid TypeMeta {:?}", obj);
			};

			// TODO: After applying a CRD, we need to find a way to wait until k8s enables its API
			let name = obj.name_any();
			if let Some((ar, caps)) = discovery.resolve_gvk(&gvk) {
				let api = dynamic_api(ar, caps, client.clone());
				let data: serde_json::Value = serde_json::to_value(&obj)?;
				let _r = api.patch(&name, &ssapply, &Patch::Apply(data)).await?;
			}
		}

		Ok(())
	}

	pub async fn delete(self) -> Result<()> {
		let client = Client::try_default().await?;
		let discovery = Discovery::new(client.clone()).run().await?;

		let mut manifests = self.manifests;
		manifests.reverse();
		for Manifest(_, doc) in manifests {
			let obj: DynamicObject = serde_json::from_value(doc)?;
			let gvk = if let Some(tm) = &obj.types {
				GroupVersionKind::try_from(tm)?
			} else {
				bail!("cannot apply object without valid TypeMeta {:?}", obj);
			};

			let name = obj.name_any();
			if let Some((ar, caps)) = discovery.resolve_gvk(&gvk) {
				let api = dynamic_api(ar, caps, client.clone());
				let _r = api.delete(&name, &Default::default()).await?;
			}
		}

		Ok(())
	}
}

impl From<Kube> for Vec<Manifest> {
	fn from(val: Kube) -> Self {
		val.manifests
	}
}

fn dynamic_api(ar: ApiResource, caps: ApiCapabilities, client: Client) -> Api<DynamicObject> {
	if caps.scope == Scope::Cluster {
		Api::all_with(client, &ar)
	} else {
		Api::default_namespaced_with(client, &ar)
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
		let filter = Filter {
			only: self.only,
			except: self.except,
		};

		let manifests = ingest::process(&value, &filter)?;

		Ok(Kube { manifests })
	}
}
