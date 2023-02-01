use kct_helper::io::Error as IOError;
use kct_kube::Error as KubeError;
use kct_package::Error as PackageError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
	#[error("Failed to parse input({0}): {1}")]
	InvalidInput(String, String),
	#[error("Malformed input - {0}")]
	MalformedInput(String),
	#[error("Output is invalid: {0}")]
	InvalidOutput(String),
	#[error(transparent)]
	IO(#[from] IOError),
	#[error(transparent)]
	InvalidPackage(#[from] PackageError),
	#[error(transparent)]
	InvalidManifest(#[from] KubeError),
}
