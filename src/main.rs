use indicatif::{ProgressBar, ProgressState, ProgressStyle};
use std::fs::create_dir_all;
use std::fmt::Write;

use rustofluid::log::*;
use rustofluid::scene::setup::{parse_args, setup_scene, CLIArgs};
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
    let (log, switch) = create_logger();

    assert_output_path(&cli.output);

    let dt = cli.dt;
    let n_steps = (cli.time_end / dt) as u64;

    let mut progress = None;

    if cli.show_progress {
        switch.disable();

        let pb = ProgressBar::new(n_steps);
        pb.set_style(
            ProgressStyle::with_template(
                "{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] ({eta})",
            )
            .unwrap()
            .with_key("eta", |state: &ProgressState, w: &mut dyn Write| {
                write!(w, "{:.1}s", state.eta().as_secs_f64()).unwrap()
            })
            .progress_chars("#>-"),
        );
        progress = Some(pb);
    }

    let mut timestepper = setup_scene(&log, &cli)?;

    for step in 0..n_steps {
        timestepper.compute_step(dt);

        save_plots(
            &log,
            &timestepper,
            cli.plot_dim,
            &cli.output,
            step,
            cli.plot_pressure,
            cli.plot_velocity,
        )?;

        if let Some(ref p) = progress {
          p.inc(1);
        }
    }

    return Ok(());
}
