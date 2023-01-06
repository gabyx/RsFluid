use crate::log::*;
use crate::plotting;
use crate::scene::grid::{CellGetter, CellTypes, Grid};
use crate::scene::timestepper::TimeStepper;
use crate::types::*;
use std::error::Error;

pub fn save_plots(
    log: &Logger,
    timestepper: &TimeStepper,
    output: &str,
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

    let v_range = grid.stats[1].velocity_norm - grid.stats[0].velocity_norm;
    let vel_get = |idx: Index2| {
        if grid.cell(idx).mode == CellTypes::Solid {
            return None;
        }
        return Some(grid.cell(idx).velocity.back.norm() - grid.stats[0].velocity_norm / v_range);
    };

    let smoke_get = |idx: Index2| {
        if grid.cell(idx).mode == CellTypes::Solid {
            return None;
        }
        return Some(0.7 * grid.cell(idx).smoke.back);
    };

    let text = format!(
        "frame: {:5.0}, pressure: [{:.3} , {:.3}], div: [{:.3} , {:.3}], vel: [{:.3} , {:.3}]",
        step,
        grid.stats[0].pressure,
        grid.stats[1].pressure,
        grid.stats[0].div,
        grid.stats[1].div,
        grid.stats[0].velocity_norm,
        grid.stats[1].velocity_norm
    );

    info!(log, "Saving plots.");

    let mut file = output.replace("{}", &format!("vel-{:06}", step));
    plotting::grid(dim!(1600, 800), grid.dim, vel_get, file, None, &text)?;

    file = output.replace("{}", &format!("press-{:06}", step));
    plotting::grid(dim!(1600, 800), grid.dim, press_get, file, None, &text)?;

    file = output.replace("{}", &format!("smoke-{:06}", step));
    return plotting::grid(dim!(1600, 800), grid.dim, smoke_get, file, None, &text);
}
