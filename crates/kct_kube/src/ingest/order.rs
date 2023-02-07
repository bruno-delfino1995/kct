use super::path;

use crate::error::{self};

use std::cmp::Ordering;
use std::path::PathBuf;

use anyhow::Result;

use serde_json::Value;

const KIND_ORDER: [&str; 35] = [
	"Namespace",
	"NetworkPolicy",
	"ResourceQuota",
	"LimitRange",
	"PodSecurityPolicy",
	"PodDisruptionBudget",
	"ServiceAccount",
	"Secret",
	"SecretList",
	"ConfigMap",
	"StorageClass",
	"PersistentVolume",
	"PersistentVolumeClaim",
	"CustomResourceDefinition",
	"ClusterRole",
	"ClusterRoleList",
	"ClusterRoleBinding",
	"ClusterRoleBindingList",
	"Role",
	"RoleList",
	"RoleBinding",
	"RoleBindingList",
	"Service",
	"DaemonSet",
	"Pod",
	"ReplicationController",
	"ReplicaSet",
	"Deployment",
	"HorizontalPodAutoscaler",
	"StatefulSet",
	"Job",
	"CronJob",
	"IngressClass",
	"Ingress",
	"APIService",
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Kind(String);

impl Kind {
	fn priority(&self) -> usize {
		KIND_ORDER
			.iter()
			.position(|&k| k == self.0)
			.unwrap_or_else(|| KIND_ORDER.len())
	}
}

impl Ord for Kind {
	fn cmp(&self, other: &Self) -> Ordering {
		self.priority().cmp(&other.priority())
	}
}

impl PartialOrd for Kind {
	fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
		Some(self.cmp(other))
	}
}

impl TryFrom<&Value> for Kind {
	type Error = error::Object;

	fn try_from(value: &Value) -> Result<Self, Self::Error> {
		let kind = value
			.get("kind")
			.and_then(|v| v.as_str())
			.map(|k| k.to_string())
			.ok_or(error::Object::NoKind)?;

		Ok(Kind(kind))
	}
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Track {
	pub field: String,
	pub depth: usize,
	pub order: usize,
	pub kind: Option<Kind>,
}

impl ToString for Track {
	fn to_string(&self) -> String {
		let kind = self
			.kind
			.clone()
			.map(|k| k.0)
			.unwrap_or_else(|| String::from("Kind"));

		format!("{}({}:{}:{})", kind, self.field, self.depth, self.order)
	}
}

impl TryFrom<&str> for Track {
	type Error = error::Tracking;

	fn try_from(source: &str) -> Result<Self, Self::Error> {
		let parts: Vec<String> = source.split(':').map(String::from).collect();

		if let [field, depth, order] = parts.as_slice() {
			let field = field.to_string();
			if !path::is_valid(&field) {
				return Err(error::Tracking::InvalidPart("field".to_string()));
			}

			let depth = depth
				.parse()
				.map_err(|_| error::Tracking::InvalidPart("depth".to_string()))?;
			let order = order
				.parse()
				.map_err(|_| error::Tracking::InvalidPart("order".to_string()))?;

			Ok(Track {
				field,
				depth,
				order,
				kind: None,
			})
		} else {
			Err(error::Tracking::Format)
		}
	}
}

impl Ord for Track {
	fn cmp(&self, other: &Self) -> Ordering {
		let first_or_equal = |orders: &[Ordering]| -> Ordering {
			*orders
				.iter()
				.find(|&&o| o != Ordering::Equal)
				.unwrap_or(&Ordering::Equal)
		};

		let field = self.field.cmp(&other.field);
		let depth = self.depth.cmp(&other.depth);
		let order = self.order.cmp(&other.order);
		let kind = match (&self.kind, &other.kind) {
			(Some(a), Some(b)) => a.cmp(b),
			(_, _) => Ordering::Equal,
		};

		if depth == Ordering::Equal {
			first_or_equal(&[order, kind, field])
		} else {
			first_or_equal(&[order, field])
		}
	}
}

impl PartialOrd for Track {
	fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
		Some(self.cmp(other))
	}
}

#[derive(Debug, Default)]
pub struct Order(Vec<Track>);

impl TryFrom<&Value> for Order {
	type Error = error::Object;

	fn try_from(value: &Value) -> Result<Self, Self::Error> {
		let annotation = value
			.get("metadata")
			.and_then(|m| m.get("annotations"))
			.and_then(|a| a.get("kct.io/order"))
			.and_then(|o| o.as_str())
			.unwrap_or_default();

		let tracking = annotation
			.split('/')
			.into_iter()
			.filter(|s| !s.is_empty())
			.map(Track::try_from)
			.collect::<Result<Vec<Track>, _>>()?;

		Ok(Order(tracking))
	}
}

#[derive(Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct Tracking(Vec<Track>);

impl Tracking {
	pub fn depth(&self) -> usize {
		let vec = &self.0;
		let index = vec.len().saturating_sub(1);

		vec.get(index).map(|t| t.depth).unwrap_or(0)
	}

	pub fn track(&self, track: Track) -> Self {
		let mut new = self.0.clone();
		new.push(track);

		Tracking(new)
	}

	pub fn ordered(self, order: Order) -> Self {
		let mut ordered: Vec<Track> = vec![];
		let length = self.0.len();

		let mut paths = self.0.into_iter().peekable();
		let mut orders = order.0.into_iter().rev().peekable();

		let ordered = loop {
			match (paths.peek(), orders.peek()) {
				(None, _) => {
					break ordered;
				}
				(Some(p), None) => {
					ordered.push(p.clone());
					paths.next();
				}
				(Some(p), Some(o)) => {
					let same_field = o.field == p.field;
					let same_depth = (length - o.depth) == p.depth;

					let track = if same_field && same_depth {
						let track = Track {
							field: p.field.clone(),
							depth: p.depth,
							order: o.order,
							kind: p.kind.clone(),
						};

						orders.next();

						track
					} else {
						p.clone()
					};

					ordered.push(track);
					paths.next();
				}
			};
		};

		Self(ordered)
	}

	pub fn kinded(mut self, kind: Kind) -> Tracking {
		let len = self.0.len().saturating_sub(1);
		if let Some(t) = self.0.get_mut(len) {
			t.kind = Some(kind);
		}

		self
	}
}

impl From<&Tracking> for PathBuf {
	fn from(source: &Tracking) -> Self {
		let mut root = PathBuf::from("/");

		for t in source.0.iter() {
			root.push(t.field.clone());
		}

		root
	}
}
