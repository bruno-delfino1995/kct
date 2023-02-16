use crate::{Manifest, Tracked};

pub use crate::error::Root as Error;

use std::time::Duration;

use anyhow::Result;
use async_trait::async_trait;
use either::Either;
use futures::TryFutureExt;
use k8s_openapi::apiextensions_apiserver::pkg::apis::apiextensions::v1::CustomResourceDefinition as CRD;
use kube::api::{Api, DynamicObject as Dynamic, Patch, PatchParams, ResourceExt};
use kube::core::GroupVersionKind;
use kube::discovery::{Discovery, Scope};
use kube::runtime::wait::{await_condition, conditions};
use kube::Client as K8s;

pub struct Client {
	internal: K8s,
	discovery: Discovery,
}

impl Client {
	pub async fn try_new() -> Result<Self> {
		let internal = K8s::try_default().await?;
		let discovery = Discovery::new(internal.clone()).run().await?;

		Ok(Self {
			internal,
			discovery,
		})
	}

	pub async fn apply(&mut self, manifests: Vec<Manifest>) -> Result<()> {
		let plan = Plan::try_new(manifests)?;

		let ssapply = PatchParams::apply("kct-crds").force();
		let crds = plan
			.crds
			.into_iter()
			.map(|crd| crd.apply(self, &ssapply).inspect_ok(Client::applied));
		let _ = futures::future::try_join_all(crds).await?;

		self.refresh().await?;
		let ssapply = PatchParams::apply("kct-dyns").force();
		let dynamics = plan
			.dynamics
			.into_iter()
			.map(|obj| obj.apply(self, &ssapply).inspect_ok(Client::applied));
		let _ = futures::future::try_join_all(dynamics).await?;

		Ok(())
	}

	fn applied(path: &String) {
		println!("{path} created")
	}

	pub async fn delete(&mut self, mut manifests: Vec<Manifest>) -> Result<()> {
		manifests.reverse();

		let plan = Plan::try_new(manifests)?;

		let dynamics = plan
			.dynamics
			.into_iter()
			.map(|obj| obj.delete(self).inspect_ok(Client::deleted));
		let crds = plan
			.crds
			.into_iter()
			.map(|obj| obj.delete(self).inspect_ok(Client::deleted));

		let _ = futures::future::try_join_all(dynamics).await?;
		let _ = futures::future::try_join_all(crds).await?;

		Ok(())
	}

	fn deleted(path: &String) {
		println!("{path} deleted")
	}

	async fn refresh(&mut self) -> Result<()> {
		self.discovery = Discovery::new(self.internal.clone()).run().await?;

		Ok(())
	}
}

struct Plan {
	crds: Vec<Tracked<CRD>>,
	dynamics: Vec<Tracked<Dynamic>>,
}

impl Plan {
	fn try_new(manifests: Vec<Manifest>) -> Result<Self> {
		let mut crds = vec![];
		let mut dynamics = vec![];

		for Tracked(path, doc) in manifests {
			let obj: Dynamic = serde_json::from_value(doc)?;

			match try_crd(obj) {
				Either::Right(crd) => crds.push((path, crd).into()),
				Either::Left(obj) => dynamics.push((path, obj).into()),
			}
		}

		Ok(Plan { crds, dynamics })
	}
}

fn try_crd(obj: Dynamic) -> Either<Dynamic, CRD> {
	match obj.clone().try_parse() {
		Ok(crd) => Either::Right(crd),
		Err(_) => Either::Left(obj),
	}
}

#[async_trait]
trait Object {
	type Kind;

	async fn apply(self, client: &Client, params: &PatchParams) -> Result<String>;

	async fn delete(self, client: &Client) -> Result<String>;

	fn api(&self, client: &Client) -> Result<Api<Self::Kind>>;
}

#[async_trait]
impl Object for Tracked<Dynamic> {
	type Kind = Dynamic;

	async fn apply(self, client: &Client, params: &PatchParams) -> Result<String> {
		let value = self.value();
		let name = value.name_any();

		let api = self.api(client)?;
		let data = serde_json::to_value(value)?;
		let _ = api.patch(&name, params, &Patch::Apply(data)).await?;

		Ok(format!("{}", self.path().display()))
	}

	async fn delete(self, client: &Client) -> Result<String> {
		let name = self.value().name_any();
		let api = self.api(client)?;
		let _ = api.delete(&name, &Default::default()).await?;

		Ok(format!("{}", self.path().display()))
	}

	fn api(&self, client: &Client) -> Result<Api<Self::Kind>> {
		let obj = self.value();
		let gvk = obj
			.types
			.as_ref()
			.ok_or(anyhow::anyhow!(
				"cannot apply object without valid TypeMeta {:?}",
				obj
			))
			.and_then(|tm| {
				let gvk = GroupVersionKind::try_from(tm)?;

				Ok(gvk)
			})?;

		let (ar, caps) = client.discovery.resolve_gvk(&gvk).ok_or(anyhow::anyhow!(
			"unable to resolve resource and capabilities from dynamic object {:?}",
			obj
		))?;

		let client = client.internal.clone();
		if caps.scope == Scope::Cluster {
			Ok(Api::all_with(client, &ar))
		} else {
			Ok(Api::default_namespaced_with(client, &ar))
		}
	}
}

#[async_trait]
impl Object for Tracked<CRD> {
	type Kind = CRD;

	async fn apply(self, client: &Client, params: &PatchParams) -> Result<String> {
		let name = self.value().name_any();
		let patch = Patch::Apply(self.value());
		let cond = conditions::is_crd_established();

		let api = self.api(client)?;
		let apply = api
			.patch(&name, params, &patch)
			.map_err(|err| anyhow::anyhow!(err));

		let api = self.api(client)?;
		let wait = {
			let establish = await_condition(api, &name, cond).map_err(|err| anyhow::anyhow!(err));

			tokio::time::timeout(Duration::from_secs(10), establish)
				.map_err(|err| anyhow::anyhow!(err))
		};

		let _ = futures::future::try_join(apply, wait).await?;

		Ok(format!("{}", self.path().display()))
	}

	async fn delete(self, client: &Client) -> Result<String> {
		let name = self.value().name_any();
		let api = self.api(client)?;
		let _ = api.delete(&name, &Default::default()).await?;

		Ok(format!("{}", self.path().display()))
	}

	fn api(&self, client: &Client) -> Result<Api<Self::Kind>> {
		Ok(Api::all(client.internal.clone()))
	}
}
