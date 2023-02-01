use thiserror::Error;

#[derive(Error, Debug)]
pub enum Root {
	#[error("The rendered json is invalid")]
	Output(#[from] Output),
	#[error("Your object is invalid")]
	Object(#[from] Object),
}

#[derive(Error, Debug)]
pub enum Output {
	#[error("The path({0}) is invalid, it has to follow RFC 1123")]
	Path(String),
	#[error("Only objects are allowed until a manifest is found")]
	NotObject,
}

#[derive(Error, Debug)]
pub enum Object {
	#[error("There's no kind in your manifest")]
	NoKind,
	#[error("Your tracking fields are invalid")]
	Tracking(#[from] Tracking),
}

#[derive(Error, Debug)]
pub enum Tracking {
	#[error("Tracking needs to have 3 parts consisting of `field:depth:order`")]
	Format,
	#[error("Your {0} part is invalid, the types are `str:uint:uint`")]
	InvalidPart(String),
}
