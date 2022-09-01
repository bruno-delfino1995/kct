use lazy_static::lazy_static;
use regex::Regex;
use serde_json::Value;
use std::{
	cmp::Ordering,
	path::{Path, PathBuf},
};
use thiserror::Error;
use valico::json_schema::Scope;

#[derive(Error, PartialEq, Eq, Debug)]
pub enum Error {
	#[error("The rendered json is invalid")]
	Invalid,
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Default)]
pub struct Filter {
	pub only: Vec<PathBuf>,
	pub except: Vec<PathBuf>,
}

impl Filter {
	fn pass(&self, path: &Path) -> bool {
		let allow = self.only.iter().any(|allow| path.starts_with(allow));

		let disallow = self
			.except
			.iter()
			.any(|disallow| path.starts_with(disallow));

		(allow || self.only.is_empty()) && !disallow
	}
}

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
struct Kind(String);

impl Ord for Kind {
	fn cmp(&self, other: &Self) -> Ordering {
		let index_a = KIND_ORDER
			.iter()
			.position(|&k| k == self.0)
			.unwrap_or_else(|| KIND_ORDER.len());
		let index_b = KIND_ORDER
			.iter()
			.position(|&k| k == other.0)
			.unwrap_or_else(|| KIND_ORDER.len());

		index_a.cmp(&index_b)
	}
}

impl PartialOrd for Kind {
	fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
		Some(self.cmp(other))
	}
}

impl TryFrom<&Value> for Kind {
	type Error = Error;

	fn try_from(value: &Value) -> std::result::Result<Self, Self::Error> {
		let kind = value
			.get("kind")
			.and_then(|v| v.as_str())
			.map(|k| k.to_string())
			.ok_or(Error::Invalid)?;

		Ok(Kind(kind))
	}
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Track {
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
	type Error = Error;

	fn try_from(source: &str) -> std::result::Result<Self, Self::Error> {
		let parts: Vec<String> = source.split(':').map(String::from).collect();

		let field = parts.get(0).map(String::from).ok_or(Error::Invalid)?;
		let depth = parts
			.get(1)
			.map(|d| d.parse())
			.transpose()
			.map_err(|_| Error::Invalid)
			.and_then(|n| n.ok_or(Error::Invalid))?;
		let order = parts
			.get(2)
			.map(|d| d.parse())
			.transpose()
			.map_err(|_| Error::Invalid)
			.and_then(|n| n.ok_or(Error::Invalid))?;

		Ok(Track {
			field,
			depth,
			order,
			kind: None,
		})
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
struct Order(Vec<Track>);

impl TryFrom<&Value> for Order {
	type Error = Error;

	fn try_from(value: &Value) -> std::result::Result<Self, Self::Error> {
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
			.collect::<std::result::Result<Vec<Track>, Self::Error>>()?;

		Ok(Order(tracking))
	}
}

#[derive(Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
struct Tracking(Vec<Track>);

impl Tracking {
	fn depth(&self) -> usize {
		let vec = &self.0;
		let index = vec.len().saturating_sub(1);

		vec.get(index).map(|t| t.depth).unwrap_or(0)
	}

	fn track(&self, track: Track) -> Self {
		let mut new = self.0.clone();
		new.push(track);

		Tracking(new)
	}

	fn ordered(self, order: Order) -> Self {
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

	fn kinded(mut self, kind: Kind) -> Tracking {
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

pub fn find(json: &Value, filter: &Filter) -> Result<Vec<(PathBuf, Value)>> {
	let mut objects: Vec<(Tracking, Value)> = vec![];
	let mut walker: Vec<Box<dyn Iterator<Item = (Tracking, &Value)>>> =
		vec![Box::new(vec![(Tracking::default(), json)].into_iter())];

	while let Some(curr) = walker.last_mut() {
		let (tracking, json) = match curr.next() {
			Some(val) => val,
			None => {
				walker.pop();
				continue;
			}
		};

		if is_object(json) {
			let order = Order::try_from(json)?;
			let tracking = tracking.ordered(order);
			let kind = Kind::try_from(json)?;
			let tracked = tracking.kinded(kind);

			let path: PathBuf = (&tracked).into();

			if filter.pass(&path) {
				objects.push((tracked, json.to_owned()));
			}
		} else {
			match json {
				Value::Object(map) => {
					let mut members: Vec<(Tracking, &Value)> = Vec::with_capacity(map.len());

					for (k, v) in map {
						if !is_valid_path(k) {
							return Err(Error::Invalid);
						} else {
							let track = Track {
								field: k.clone(),
								depth: tracking.depth() + 1,
								order: map.len(),
								kind: None,
							};

							members.push((tracking.track(track), v))
						}
					}

					walker.push(Box::new(members.into_iter()));
				}
				_ => return Err(Error::Invalid),
			}
		}
	}

	objects.sort_by(|(a, _), (b, _)| a.cmp(b));

	Ok(objects.into_iter().map(|(t, v)| ((&t).into(), v)).collect())
}

const K8S_OBJECT_SCHEMA: &str = r#"{
	"$schema": "http://json-schema.org/schema#",
	"type": "object",
	"additionalProperties": true,
	"required": ["kind", "apiVersion"],
	"properties": {
		"kind": {
			"type": "string"
		},
		"apiVersion": {
			"type": "string"
		}
	}
}"#;

fn is_object(obj: &Value) -> bool {
	let schema = serde_json::from_str(K8S_OBJECT_SCHEMA).unwrap();

	let mut scope = Scope::new();
	let validator = scope.compile_and_return(schema, false).unwrap();

	validator.validate(obj).is_strictly_valid()
}

fn is_valid_path(path: &str) -> bool {
	lazy_static! {
		static ref PATTERN: Regex =
			Regex::new(r"(?i)^[a-z0-9]$|^[a-z0-9][a-z0-9-]*[a-z0-9]$").unwrap();
	}

	PATTERN.is_match(path)
}
