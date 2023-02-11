mod error;
mod install;
mod instrument;
mod operation;
mod render;
mod uninstall;

use anyhow::Result;
use clap::{ArgAction, Parser, Subcommand};

#[derive(Parser)]
#[command(
	version,
	about = "Kubernetes templates without hideous language or context babysitting",
	name = "Kubernetes Configuration Tool"
)]
#[command(
	disable_help_subcommand = true,
	help_expected = true,
	arg_required_else_help = true
)]
pub struct App {
	#[arg(help = "increase logging levels", long, short, global = true, action = ArgAction::Count)]
	verbose: u8,
	#[command(subcommand)]
	command: Command,
}

#[derive(Subcommand)]
pub enum Command {
	#[command(
		name = "render",
		alias = "r",
		about = "Compiles and renders your package"
	)]
	Render(render::Args),
	#[command(
		name = "install",
		alias = "i",
		about = "Puts your objects in the current cluster"
	)]
	Install(install::Args),
	#[command(
		name = "uninstall",
		alias = "u",
		about = "Removes your objects from the current cluster"
	)]
	Uninstall(uninstall::Args),
}

#[tokio::main]
async fn main() -> Result<()> {
	let app = App::parse();

	let _guard = instrument::init(app.verbose);

	match app.command {
		Command::Render(args) => render::run(args)?,
		Command::Install(args) => install::run(args).await?,
		Command::Uninstall(args) => uninstall::run(args).await?,
	};

	Ok(())
}
