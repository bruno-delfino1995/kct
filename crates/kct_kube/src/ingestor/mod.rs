mod order;
mod path;

use crate::error::{self, Root as Error};
use crate::Manifest;

use self::order::{Kind, Order, Track, Tracking};

pub use self::path::Filter;

use std::collections::BinaryHeap;
use std::path::PathBuf;

use anyhow::Result;
use serde_json::Value;

type Plate<'a> = Box<dyn Iterator<Item = (Tracking, &'a Value)> + 'a>;

struct Food(Tracking, Value);

impl From<&Food> for PathBuf {
	fn from(val: &Food) -> Self {
		(&val.0).into()
	}
}

impl From<Food> for Manifest {
	fn from(val: Food) -> Self {
		let path = (&val).into();

		(path, val.1).into()
	}
}

impl Eq for Food {}
impl PartialEq for Food {
	fn eq(&self, other: &Self) -> bool {
		self.0.eq(&other.0)
	}
}

impl Ord for Food {
	fn cmp(&self, other: &Self) -> std::cmp::Ordering {
		self.0.cmp(&other.0)
	}
}

impl PartialOrd for Food {
	fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
		Some(self.cmp(other))
	}
}

pub struct Ingestor {
	filter: Filter,
}

impl Ingestor {
	pub fn new(only: Vec<PathBuf>, except: Vec<PathBuf>) -> Self {
		let filter = Filter { only, except };

		Self { filter }
	}

	pub fn ingest<'a>(&'_ self, json: &'a Value) -> Result<Vec<Manifest>, Error> {
		let mut manifests = BinaryHeap::new();

		let root = vec![(Tracking::default(), json)];
		let mut stack: Vec<Plate<'a>> = vec![Box::new(root.into_iter())];

		// DFS with a companion tracking that inherits the path from its parent
		while let Some(curr) = stack.last_mut() {
			let (tracking, json) = match curr.next() {
				Some(val) => val,
				None => {
					stack.pop();
					continue;
				}
			};

			if Manifest::conforms(json) {
				let found = self.on_found(tracking, json)?;
				let path: PathBuf = (&found).into();

				if self.filter.pass(&path) {
					manifests.push(found)
				}
			} else {
				let to_search = self.on_search(tracking, json)?;

				stack.push(Box::new(to_search));
			}
		}

		Ok(manifests
			.into_sorted_vec()
			.into_iter()
			.map(|food| food.into())
			.collect())
	}

	fn on_found(&self, tracking: Tracking, json: &Value) -> Result<Food, Error> {
		let order = Order::try_from(json)?;
		let kind = Kind::try_from(json)?;
		let tracked = tracking.ordered(order).kinded(kind);

		Ok(Food(tracked, json.to_owned()))
	}

	fn on_search<'a>(&'_ self, tracking: Tracking, json: &'a Value) -> Result<Plate<'a>, Error> {
		let props = match json {
			Value::Object(props) => props,
			_ => return Err(error::Output::NotObject)?,
		};

		let mut members: Vec<(Tracking, &Value)> = Vec::with_capacity(props.len());
		let length = props.len();
		let depth = tracking.depth() + 1;

		for (k, v) in props {
			if path::is_valid(k) {
				let track = Track {
					depth,
					order: length,
					field: k.clone(),
					kind: None,
				};

				members.push((tracking.track(track), v))
			} else {
				return Err(error::Output::Path(k.to_string()))?;
			}
		}

		Ok(Box::new(members.into_iter()))
	}
}
