use rustofluid::log::*;

fn main() {
    let log = create_logger();

    trace!(log, "Logging ready!");
    debug!(log, "Logging ready!");
    error!(log, "Logging ready!");
}
