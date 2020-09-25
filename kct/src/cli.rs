pub mod compile;

use clap::App;

pub fn create() -> App<'static, 'static> {
	App::new("Kubernetes Configuration Tool")
		.version("0.1.0")
		.about("K8S config without hideous templates or context babysitting")
		.subcommand(compile::command())
}
