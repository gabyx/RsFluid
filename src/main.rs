
use rustofluid::log::*;
use rustofluid::solver::grid;

fn main() {
    let log = create_logger();

    trace!(log, "Logging ready!");
    debug!(log, "Logging ready!");
    error!(log, "Logging ready!");

    let g: grid::Grid<10,10> = grid::Grid::new();
}
