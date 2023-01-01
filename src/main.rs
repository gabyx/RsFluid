use std::fmt::Debug;
use std::fs::create_dir_all;
use std::str::FromStr;

use clap::Parser;
use nalgebra as na;

use rustofluid::draw::*;
use rustofluid::log::*;
use rustofluid::solver::grid::{Grid,CellGetter};
use rustofluid::solver::timestepper::{Integrate, TimeStepper};
use rustofluid::types::*;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short = 'o', long, default_value_t = String::from("./frames/frame-{}.png"))]
    output: String,

    #[arg(short = 'e', long = "time-end", default_value_t = 10.0)]
    time_end: Scalar,

    #[arg(short = 't', long = "timestep", default_value_t = 0.001)]
    timestep: Scalar,

    #[arg(long = "density", default_value_t = 1.0)]
    density: Scalar,

    #[arg(long = "dim", default_value = "100, 100", value_parser = parse_vector::<usize, 2>)]
    dim: Index2,

    #[arg(short = 'g', long = "gravity", default_value = "0.0, 9.81",  value_parser = parse_vector::<Scalar, 2>)]
    gravity: Vector2,

    #[arg(long = "incompress-iters", default_value_t = 10)]
    incompress_iter: u64,
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

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let log = create_logger();

    let cli = Args::parse();

    std::path::Path::new(&cli.output)
        .parent()
        .and_then(|p| Some(create_dir_all(p).expect("Could not create dir.")));

    let grid = Grid::new(cli.dim, 1.0);
    let objs: Vec<Box<dyn Integrate>> = vec![Box::new(grid)];

    let mut timestepper =
        TimeStepper::new(&log, cli.density, cli.gravity, cli.incompress_iter, objs);

    let dt = cli.timestep;
    let t_end = 2.0;
    let n_steps = (t_end / dt) as u64;

    let vel_get = |i, j| {
        return grid.cell(idx!(i, j)).velocity.back.norm();
    };

    for _ in 0..n_steps {
        timestepper.compute_step(dt);

        let file = std::fmt::format(format_args!(cli.output, n_steps));
        plot::grid(dim, vel_get, );
    }

    return Ok(());
}
