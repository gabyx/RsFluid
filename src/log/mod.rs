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
