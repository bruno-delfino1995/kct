use crate::operation::compile;

use anyhow::Result;
use clap::Parser;

#[derive(Parser)]
pub struct Args {
	#[command(flatten)]
	compile: compile::Params,
}

pub async fn run(args: Args) -> Result<String> {
	let kube = compile::run(args.compile)?;
	kube.install().await?;

	Ok("success".to_string())
}
