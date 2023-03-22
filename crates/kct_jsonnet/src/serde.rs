use std::convert::From;
use std::convert::{TryFrom, TryInto};

use jrsonnet_evaluator::error::Error;
use jrsonnet_evaluator::error::ErrorKind;
use jrsonnet_evaluator::val::StrValue;
use jrsonnet_evaluator::ObjValueBuilder;
use jrsonnet_evaluator::Val;
use serde_json::{Map, Number, Value};

pub struct W<T>(pub T);

impl TryFrom<W<&Val>> for Value {
	type Error = Error;
	fn try_from(wrapper: W<&Val>) -> jrsonnet_evaluator::Result<Self> {
		let v = wrapper.0;

		Ok(match v {
			Val::Bool(b) => Self::Bool(*b),
			Val::Null => Self::Null,
			Val::Str(s) => Self::String(s.to_string()),
			Val::Num(n) => Self::Number(if n.fract() <= f64::EPSILON {
				(*n as i64).into()
			} else {
				Number::from_f64(*n).expect("to json number")
			}),
			Val::Arr(a) => {
				let mut out = Vec::with_capacity(a.len());
				for item in a.iter() {
					let item = item?;
					let w = W(&item);
					out.push(w.try_into()?);
				}
				Self::Array(out)
			}
			Val::Obj(o) => {
				let mut out = Map::new();
				for key in o.fields() {
					let val = o.get(key.clone())?.expect("field exists");
					let w = W(&val);

					out.insert((&key as &str).into(), w.try_into()?);
				}
				Self::Object(out)
			}
			Val::Func(_) => {
				return Err(Error::new(ErrorKind::RuntimeError(
					"tried to manifest function".into(),
				)))
			}
		})
	}
}

impl From<&Value> for W<Val> {
	fn from(v: &Value) -> Self {
		let val = match v {
			Value::Null => Val::Null,
			Value::Bool(v) => Val::Bool(*v),
			Value::Number(n) => Val::Num(n.as_f64().expect("as f64")),
			Value::String(s) => Val::Str(StrValue::Flat(s.into())),
			Value::Array(a) => {
				let mut out: Vec<Val> = Vec::with_capacity(a.len());
				for v in a {
					let w: W<Val> = v.into();
					out.push(w.0);
				}
				Val::Arr(out.into())
			}
			Value::Object(o) => {
				let mut builder = ObjValueBuilder::with_capacity(o.len());
				for (k, v) in o {
					let w: W<Val> = v.into();
					let key = (k as &str).into();
					builder.member(key).value_unchecked(w.0);
				}
				Val::Obj(builder.build())
			}
		};

		W(val)
	}
}
