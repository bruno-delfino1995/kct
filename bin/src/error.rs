use kct_helper::io::Error as IOError;
use kct_kube::Error as KubeError;
use kct_package::Error as PackageError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
	#[error("Invalid input: {0}")]
	InvalidInput(String),
	#[error(transparent)]
	IO(#[from] IOError),
	#[error(transparent)]
	InvalidPackage(#[from] PackageError),
	#[error(transparent)]
	InvalidManifest(#[from] KubeError),
}
