use slog::{o, Drain};
use slog_async;
use std::io;

fn no_out(_io: &mut dyn io::Write) -> io::Result<()> {
    return Ok(());
}

pub fn create_logger() -> slog::Logger {
    let decorator = slog_term::TermDecorator::new().build();
    let fmt = slog_term::FullFormat::new(decorator)
        .use_custom_timestamp(no_out)
        .build()
        .fuse();
    let drain = slog_async::Async::new(fmt).build().fuse();

    return slog::Logger::root(drain, o!());
}

/// Log trace level record
#[macro_export]
macro_rules! log_trace(
    ($l:expr, #$tag:expr, $($args:tt)+) => {
        slog::log!($l, slog::Level::Trace, $tag, $($args)+)
    };
    ($l:expr, $($args:tt)+) => {
        slog::log!($l, slog::Level::Trace, "", $($args)+)
    };
);

pub use log_trace as trace;

/// Log debug level record
#[macro_export]
macro_rules! log_debug(
    ($l:expr, #$tag:expr, $($args:tt)+) => {
        slog::log!($l, slog::Level::Debug, $tag, $($args)+)
    };
    ($l:expr, $($args:tt)+) => {
        slog::log!($l, slog::Level::Debug, "", $($args)+)
    };
);

pub use log_debug as debug;

/// Log info level record
#[macro_export]
macro_rules! log_info(
    ($l:expr, #$tag:expr, $($args:tt)+) => {
        slog::log!($l, slog::Level::Info, $tag, $($args)+)
    };
    ($l:expr, $($args:tt)+) => {
        slog::log!($l, slog::Level::Info, "", $($args)+)
    };
);

pub use log_info as info;

/// Log warn level record
#[macro_export]
macro_rules! log_warn(
    ($l:expr, #$tag:expr, $($args:tt)+) => {
        slog::log!($l, slog::Level::Warn, $tag, $($args)+)
    };
    ($l:expr, $($args:tt)+) => {
        slog::log!($l, slog::Level::Warn, "", $($args)+)
    };
);

pub use log_warn as warn;

/// Log warn level record
#[macro_export]
macro_rules! log_error(
    ($l:expr, #$tag:expr, $($args:tt)+) => {
        slog::log!($l, slog::Level::Error, $tag, $($args)+)
    };
    ($l:expr, $($args:tt)+) => {
        slog::log!($l, slog::Level::Error, "", $($args)+)
    };
);

pub use log_error as error;
