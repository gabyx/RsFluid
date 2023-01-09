use std::fmt::Debug;
use std::str::FromStr;

use crate::log::*;
use crate::scene::grid::{CellGetter, CellTypes, Grid};
use crate::scene::timestepper::{Integrate, Manipulator, TimeStepper};
use crate::types::*;
use clap::Parser;
use nalgebra as na;

fn parse_vector<T, const DIM: usize>(s: &str) -> Result<na::SVector<T, DIM>, String>
where
    T: na::Scalar + FromStr,
    <T as FromStr>::Err: Debug,
{
    let ss = s.split(',').collect::<Vec<&str>>();

    if ss.len() != DIM {
        return Err(format!("Need {} comma-separated values. {:?}", DIM, ss));
    }

    let it = (0..DIM).into_iter().map(|i| return ss[i]).map(|s| {
        return s
            .trim()
            .parse::<T>()
            .expect(&format!("Value '{}' is not a number.", s));
    });
    return Ok(na::SVector::<T, DIM>::from_iterator(it));
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct CLIArgs {
    #[arg(short = 'o', long, default_value_t = String::from("./frames/frame-{}.png"))]
    pub output: String,

    #[arg(short = 'e', long = "time-end", default_value_t = 5.0)]
    pub time_end: Scalar,

    #[arg(short = 't', long = "timestep", default_value_t = 0.016)]
    pub dt: Scalar,

    #[arg(long = "density", default_value_t = 1000.0)]
    pub density: Scalar,

    #[arg(long = "dim", default_value = "200, 100", value_parser = parse_vector::<usize, 2>)]
    pub dim: Index2,

    #[arg(long = "plot-dim", default_value = "1600, 800", value_parser = parse_vector::<usize, 2>)]
    pub plot_dim: Index2,

    #[arg(short = 'g', long = "gravity", default_value = "0.0, 9.81",  value_parser = parse_vector::<Scalar, 2>)]
    pub gravity: Vector2,

    #[arg(long = "incompress-iters", default_value_t = 100)]
    pub incompress_iter: u64,

    #[arg(long = "scene-index", default_value_t = 0)]
    pub scene_idx: usize,

    #[arg(long = "plot-pressure", default_value_t = false)]
    pub plot_pressure: bool,

    #[arg(long = "plot-velocity", default_value_t = false)]
    pub plot_velocity: bool,

    #[arg(long = "plot-masked-pressure", default_value_t = false)]
    pub plot_masked_pressure: bool,

    #[arg(long = "plot-masked-velocity", default_value_t = false)]
    pub plot_masked_velocity: bool,

    #[arg(long = "plot-stats", default_value_t = false)]
    pub plot_stats: bool,

    #[arg(long = "show-progress", default_value_t = false)]
    pub show_progress: bool,
}

pub fn parse_args() -> CLIArgs {
    return CLIArgs::parse();
}

struct AddSmokeBar {
    pub center: Index2,
    pub height: usize,
}

impl Manipulator for AddSmokeBar {
    fn manipulate(
        &self,
        log: &Logger,
        t: Scalar,
        dt: Scalar,
        objects: &mut Vec<Box<dyn Integrate>>,
    ) {
        debug!(log, "Add smoke at {}, {}", t, dt);

        let grid = objects
            .get_mut(0)
            .expect("No objects.")
            .as_mut()
            .as_any_mut()
            .downcast_mut::<Grid>()
            .expect("");

        // Setup smoke on border.
        let y_range = [
            self.center.y - (self.height / 2),
            self.center.y + (self.height / 2),
        ];

        (y_range[0]..y_range[1]).for_each(|y| {
            let idx = idx!(0, y);
            if let Some(cell) = grid.cell_mut_opt(idx) {
                cell.smoke.back = 1.0;
            }
        });
    }
}

pub fn setup_scene<'t>(log: &'t Logger, cli: &'t CLIArgs) -> SimpleResult<Box<TimeStepper<'t>>> {
    let velocity_in = vec2!(2.0, 0.0);
    let height = 1.0;
    let cell_width = height / cli.dim.y as Scalar;
    let width = cli.dim.x as Scalar * cell_width;

    let obstacle_size_rel = 0.3;
    let obstacle_size = obstacle_size_rel * height;

    info!(
        log,
        "Grid: {} x {}, [dim-x: {}, dim-y: {}, cell-width: {}]",
        width,
        height,
        cli.dim.x,
        cli.dim.y,
        cell_width
    );

    let mut grid = Box::new(Grid::new(cli.dim, cell_width));

    if cli.scene_idx == 0 {
        for idx in grid.iter_index() {
            let is_inside = grid.is_inside_border(idx);

            // Set walls.
            if idx.x == 0 || idx.y == 0 || idx.y == grid.dim.y - 1 {
                grid.cell_mut(idx).mode = CellTypes::Solid;
            }

            if is_inside && idx.x == 1 {
                grid.cell_mut(idx).velocity.back = velocity_in;
            }
        }
    } else {
        bail!("Not implemented scene index '{}'.", cli.scene_idx);
    }

    let grav = if cli.scene_idx == 0 {
        Vector2::zeros()
    } else {
        cli.gravity
    };

    // Setup obstacle.
    let p = vec2!(width * 0.25, height * 0.5);
    grid.set_obstacle(p, obstacle_size / 2.0, None);


    // Set manipulator (for smoke).
    let smoke_adder = Box::new(AddSmokeBar {
        center: idx!(1, grid.dim.y / 2),
        height: (1.1 * obstacle_size_rel * grid.dim.y as Scalar) as usize,
    });

    let manips: Vec<Box<dyn Manipulator>> = vec![smoke_adder];
    let objs: Vec<Box<dyn Integrate>> = vec![grid];

    let timestepper = Box::new(TimeStepper::new(
        &log,
        cli.density,
        grav,
        cli.incompress_iter,
        objs,
        manips,
    ));

    return Ok(timestepper);
}
