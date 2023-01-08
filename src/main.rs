use indicatif::{ProgressBar, ProgressState, ProgressStyle};
use std::fmt::Write;
use std::fs::create_dir_all;

use rustofluid::log::*;
use rustofluid::scene::setup::{parse_args, setup_scene, CLIArgs};
use rustofluid::scene::visualization::{save_plots, PlotParams, PlotParamsBuilder};
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

fn create_progressbar(steps: u64) -> ProgressBar {
    let pb = ProgressBar::new(steps);
    pb.set_style(
        ProgressStyle::with_template(
            "{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {pos:>7}/{len:7} ({eta})",
        )
        .unwrap()
        .with_key("eta", |state: &ProgressState, w: &mut dyn Write| {
            write!(w, "{:.1}s", state.eta().as_secs_f64()).unwrap()
        })
        .progress_chars("#>-"),
    );
    return pb;
}

fn create_plot_params(cli: &CLIArgs) -> PlotParams {
    return PlotParamsBuilder::default()
        .with_pressure(cli.plot_pressure)
        .with_velocity(cli.plot_velocity)
        .output(cli.output.clone())
        .size(cli.plot_dim)
        .with_stats(cli.plot_stats)
        .with_velocity_masked(cli.plot_masked_velocity)
        .with_pressure_masked(cli.plot_masked_pressure)
        .build()
        .unwrap();
}

fn run(cli: &CLIArgs) -> GenericResult<()> {
    let (log, switch) = create_logger();

    assert_output_path(&cli.output);

    let dt = cli.dt;
    let n_steps = (cli.time_end / dt) as u64;

    let mut progress = None;

    if cli.show_progress {
        switch.disable();
        progress = Some(create_progressbar(n_steps));
    }

    let mut timestepper = setup_scene(&log, &cli)?;
    let plot_params = create_plot_params(&cli);

    for step in 0..n_steps {
        timestepper.compute_step(dt);

        save_plots(&log, &timestepper, step, &plot_params)?;

        if let Some(ref p) = progress {
            p.inc(1);
        }
    }

    return Ok(());
}
