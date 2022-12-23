use rustofluid::log::{create_logger};
use slog::{info, debug};

fn main() {
    let log = create_logger();
    info!(log, "Logging ready!");
    debug!(log, "Logging ready!");
}
