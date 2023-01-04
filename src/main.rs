use std::error::Error;
use std::fmt::Debug;
use std::fs::create_dir_all;
use std::str::FromStr;

use clap::Parser;
use nalgebra as na;

use rustofluid::draw::*;
use rustofluid::log::*;
use rustofluid::solver::grid::CellTypes;
use rustofluid::solver::grid::{CellGetter, Grid};
use rustofluid::solver::timestepper::{Integrate, TimeStepper};
use rustofluid::types::*;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct CLIArgs {
    #[arg(short = 'o', long, default_value_t = String::from("./frames/frame-{}.png"))]
    output: String,

    #[arg(short = 'e', long = "time-end", default_value_t = 0.02)]
    time_end: Scalar,

    #[arg(short = 't', long = "timestep", default_value_t = 0.016)]
    dt: Scalar,

    #[arg(long = "density", default_value_t = 1000.0)]
    density: Scalar,

    #[arg(long = "dim", default_value = "200, 100", value_parser = parse_vector::<usize, 2>)]
    dim: Index2,

    #[arg(short = 'g', long = "gravity", default_value = "0.0, 9.81",  value_parser = parse_vector::<Scalar, 2>)]
    gravity: Vector2,

    #[arg(long = "incompress-iters", default_value_t = 40)]
    incompress_iter: u64,

    #[arg(long = "scene-index", default_value_t = 0)]
    scene_idx: usize,

    #[arg(long = "video", default_value_t = true)]
    render_video: bool,
}

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

fn setup_scene<'t>(log: &'t Logger, cli: &'t CLIArgs) -> SimpleResult<Box<TimeStepper<'t>>> {
    let mut grid = Box::new(Grid::new(cli.dim, 1.0));

    if cli.scene_idx == 0 {
        for idx in grid.iter_index() {
            // Set walls.
            if idx.x == 0 || idx.y == 0 || idx.y == grid.dim.y - 1 {
                grid.cell_mut(idx).mode = CellTypes::Solid;
            }

            if grid.is_inside_border(idx) && idx.x == 1 {
                grid.cell_mut(idx).velocity.back = vec2!(15.0, 0.0);
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

    let p = idx!(grid.dim.x / 4, grid.dim.y / 2);
    grid.set_obstacle(p.cast::<Scalar>(), 15.0);

    let objs: Vec<Box<dyn Integrate>> = vec![grid];

    let timestepper = Box::new(TimeStepper::new(
        &log,
        cli.density,
        grav,
        cli.incompress_iter,
        objs,
    ));

    return Ok(timestepper);
}

fn assert_output_path(output: &str) {
    std::path::Path::new(&output)
        .parent()
        .and_then(|p| Some(create_dir_all(p).unwrap()));
}

fn main() -> GenericResult<()> {
    let cli = CLIArgs::parse();
    return run(&cli);
}

fn run(cli: &CLIArgs) -> GenericResult<()> {
    let log = create_logger();

    assert_output_path(&cli.output);

    let mut timestepper = setup_scene(&log, &cli)?;

    let dt = cli.dt;
    let n_steps = (cli.time_end / dt) as u64;

    for step in 0..n_steps {
        timestepper.compute_step(dt);

        plot(&log, &timestepper, &cli, step)?;
    }

    return Ok(());
}

fn plot(
    log: &Logger,
    timestepper: &TimeStepper,
    cli_args: &CLIArgs,
    step: u64,
) -> Result<(), Box<dyn Error>> {
    let grid = timestepper.objects[0]
        .as_any()
        .downcast_ref::<Grid>()
        .expect("Not a grid");

    let p_range = grid.stats[1].pressure - grid.stats[0].pressure;
    let press_get = |idx: Index2| {
        if grid.cell(idx).mode == CellTypes::Solid {
            return None;
        }
        return Some((grid.cell(idx).pressure - grid.stats[0].pressure) / p_range);
    };

    let vel_get = |idx: Index2| {
        if grid.cell(idx).mode == CellTypes::Solid {
            return None;
        }
        return Some(grid.cell(idx).velocity.back.norm() / 5.0);
    };

    let text = format!(
        "frame: {:5.0}, pressure: [{:.3} , {:.3}], div: [{:.3} , {:.3}]",
        step, grid.stats[0].pressure, grid.stats[1].pressure, grid.stats[0].div, grid.stats[1].div
    );

    info!(log, "Saving plots.");

    let mut file = cli_args.output.replace("{}", &format!("vel-{:06}", step));
    plot::grid(dim!(800, 600), grid.dim, vel_get, file, None, &text)?;

    file = cli_args.output.replace("{}", &format!("press-{:06}", step));
    return plot::grid(dim!(800, 600), grid.dim, press_get, file, None, &text);
}
