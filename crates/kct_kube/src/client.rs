use crate::{Manifest, Tracked};

pub use crate::error::Root as Error;

use std::time::Duration;

use anyhow::Result;
use either::Either;
use futures::TryFutureExt;
use k8s_openapi::apiextensions_apiserver::pkg::apis::apiextensions::v1::CustomResourceDefinition as CRD;
use kube::api::{Api, DynamicObject as Dynamic, Patch, PatchParams, ResourceExt};
use kube::core::GroupVersionKind;
use kube::discovery::{Discovery, Scope};
use kube::runtime::wait::{await_condition, conditions};
use kube::Client as K8;

pub struct Client {
	client: K8,
	discovery: Discovery,
}

impl Client {
	pub async fn try_new() -> Result<Self> {
		let client = K8::try_default().await?;
		let discovery = Discovery::new(client.clone()).run().await?;

		Ok(Self { client, discovery })
	}

	pub async fn apply(&mut self, manifests: Vec<Manifest>) -> Result<()> {
		let plan = Plan::try_new(manifests)?;

		let ssapply = PatchParams::apply("kct-crds").force();
		let crds = plan
			.crds
			.into_iter()
			.map(|crd| self.apply_crd(crd, &ssapply));
		let _ = futures::future::try_join_all(crds).await?;

		self.refresh().await?;
		let ssapply = PatchParams::apply("kct-dyns").force();
		let dynamics = plan
			.dynamics
			.into_iter()
			.map(|obj| self.apply_dynamic(obj, &ssapply));
		let _ = futures::future::try_join_all(dynamics).await?;

		Ok(())
	}

	async fn apply_crd(&self, tracked: Tracked<CRD>, params: &PatchParams) -> Result<()> {
		let name = tracked.value().name_any();
		let patch = Patch::Apply(tracked.value());
		let cond = conditions::is_crd_established();

		let api: Api<CRD> = Api::all(self.client.clone());
		let apply = api
			.patch(&name, params, &patch)
			.map_err(|err| anyhow::anyhow!(err));

		let api: Api<CRD> = Api::all(self.client.clone());
		let wait = {
			let establish = await_condition(api, &name, cond).map_err(|err| anyhow::anyhow!(err));

			tokio::time::timeout(Duration::from_secs(10), establish)
				.map_err(|err| anyhow::anyhow!(err))
		};

		let _ = futures::future::try_join(apply, wait).await?;

		println!("{} created", tracked.path().display());

		Ok(())
	}

	async fn apply_dynamic(&self, tracked: Tracked<Dynamic>, params: &PatchParams) -> Result<()> {
		let name = tracked.value().name_any();
		let api: Api<Dynamic> = self.dynamic_api(tracked.value())?;
		let data = serde_json::to_value(tracked.value())?;
		let _ = api.patch(&name, params, &Patch::Apply(data)).await?;

		println!("{} created", tracked.path().display());

		Ok(())
	}

	pub async fn delete(&mut self, mut manifests: Vec<Manifest>) -> Result<()> {
		manifests.reverse();

		let plan = Plan::try_new(manifests)?;

		let dynamics = plan
			.dynamics
			.into_iter()
			.map(|obj| self.delete_dynamic(obj));
		let crds = plan.crds.into_iter().map(|obj| self.delete_crd(obj));

		let _ = futures::future::try_join_all(dynamics).await?;
		let _ = futures::future::try_join_all(crds).await?;

		Ok(())
	}

	async fn delete_crd(&self, crd: Tracked<CRD>) -> Result<()> {
		let name = crd.value().name_any();
		let api: Api<CRD> = Api::all(self.client.clone());
		let _ = api.delete(&name, &Default::default()).await?;

		println!("{} removed", crd.path().display());

		Ok(())
	}

	async fn delete_dynamic(&self, dynamic: Tracked<Dynamic>) -> Result<()> {
		let name = dynamic.value().name_any();
		let api: Api<Dynamic> = self.dynamic_api(dynamic.value())?;
		let _ = api.delete(&name, &Default::default()).await?;

		println!("{} removed", dynamic.path().display());

		Ok(())
	}

	async fn refresh(&mut self) -> Result<()> {
		self.discovery = Discovery::new(self.client.clone()).run().await?;

		Ok(())
	}

	fn dynamic_api(&self, obj: &Dynamic) -> Result<Api<Dynamic>> {
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

		let (ar, caps) = self.discovery.resolve_gvk(&gvk).ok_or(anyhow::anyhow!(
			"unable to resolve resource and capabilities from dynamic object {:?}",
			obj
		))?;

		let client = self.client.clone();
		if caps.scope == Scope::Cluster {
			Ok(Api::all_with(client, &ar))
		} else {
			Ok(Api::default_namespaced_with(client, &ar))
		}
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
