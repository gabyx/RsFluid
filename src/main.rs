use rustofluid::log::*;
use rustofluid::solver::grid::Grid;
use rustofluid::solver::timestepper::{Integrate, TimeStepper};
use rustofluid::types::*;

fn main() {
    let log = create_logger();

    let dt = 0.001;
    let density = 1.0;
    let gravity = Vector2::new(0.0, -9.81);

    let objs: Vec<Box<dyn Integrate>> = vec![Box::new(Grid::new(100, 100, 1.0))];

    let mut timestepper = TimeStepper::new(&log, density, gravity, 10, objs);

    let t_end = 2.0;

    let n_steps = (t_end / dt) as u64;

    for _ in 0..n_steps {
        timestepper.compute_step(dt);
    }
}
