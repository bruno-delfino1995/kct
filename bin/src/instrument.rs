use tracing::error;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::filter::LevelFilter;
use tracing_subscriber::fmt;
use tracing_subscriber::{layer::SubscriberExt, Registry};

pub struct Guard {
	_appender: WorkerGuard,
}

#[must_use]
pub fn init(level: u8) -> Guard {
	let (writer, guard) = tracing_appender::non_blocking(std::io::stderr());

	let logs = fmt::layer()
		.with_file(true)
		.with_target(true)
		.with_level(true)
		.with_line_number(true)
		.with_ansi(true)
		.with_thread_ids(true)
		.with_writer(writer);

	let max_level = match level {
		0 => LevelFilter::ERROR,
		1 => LevelFilter::WARN,
		2 => LevelFilter::INFO,
		3 => LevelFilter::DEBUG,
		_ => LevelFilter::TRACE,
	};

	let subscriber = Registry::default().with(max_level).with(logs);

	tracing::subscriber::set_global_default(subscriber)
		.expect("Unable to register tracing subscriber");

	std::panic::set_hook(Box::new(|info| {
		let message = match info.payload().downcast_ref::<&str>() {
			Some(msg) => msg.to_string(),
			None => String::from("CLI Crashed"),
		};

		let (file, line) = match info.location() {
			Some(location) => (Some(location.file()), Some(location.line())),
			None => (None, None),
		};

		error!(message, panic = true, panic.file = file, panic.line = line)
	}));

	Guard { _appender: guard }
}
