use std::fs::create_dir_all;

use rustofluid::log::*;
use rustofluid::scene::setup::{setup_scene, CLIArgs, parse_args};
use rustofluid::scene::visualization::save_plots;
use rustofluid::types::*;

fn assert_output_path(output: &str) {
    std::path::Path::new(&output)
        .parent()
        .and_then(|p| Some(create_dir_all(p).unwrap()));
}

fn main() -> GenericResult<()> {
    let cli = parse_args();
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

        save_plots(&log, &timestepper, &cli.output, step)?;
    }

    return Ok(());
}
