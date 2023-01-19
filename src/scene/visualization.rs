use crate::log::*;
use crate::plotting;
use crate::plotting::ColorFunction;
use crate::scene::grid::{CellGetter, Grid};
use crate::scene::cell::CellTypes;
use crate::scene::timestepper::TimeStepper;
use crate::types::*;
use colorgrad;
use std::error::Error;

#[derive(Builder)]
#[builder(pattern = "mutable")]
pub struct PlotParams {
    #[builder(default)]
    pub with_pressure: bool,

    #[builder(default)]
    pub with_velocity: bool,

    #[builder(default)]
    pub with_stats: bool,

    #[builder(default)]
    pub with_velocity_masked: bool, // Masked by smoke advection values.

    #[builder(default)]
    pub with_pressure_masked: bool,

    #[builder(default = "idx!(800,400)")]
    pub size: Index2,

    #[builder(default)]
    pub output: String,
}

fn make_solid<'a>(
    grid: &'a Grid,
    solid_color: &'a colorgrad::Color,
    f: &'a impl ColorFunction,
) -> impl ColorFunction + 'a {
    return |idx: Index2| {
        if grid.cell(idx).mode == CellTypes::Solid {
            return solid_color.clone();
        }
        return f(idx);
    };
}

fn make_masked<'a>(grid: &'a Grid, f: &'a impl ColorFunction) -> impl ColorFunction + 'a {
    return |idx: Index2| {
        let mut c = f(idx);
        c.a *= grid.cell(idx).smoke.back;
        return c;
    };
}

pub fn save_plots(
    log: &Logger,
    timestepper: &TimeStepper,
    step: u64,
    params: &PlotParams,
) -> Result<(), Box<dyn Error>> {
    info!(log, "Saving plots.");

    let grid = timestepper.objects[0]
        .as_any()
        .downcast_ref::<Grid>()
        .expect("Not a grid");

    let cg: colorgrad::Gradient = colorgrad::turbo();
    let solid_color = colorgrad::Color::new(0.2, 0.2, 0.2, 1.0);

    let text = if params.with_stats {
        Some(format!(
            "frame: {:5.0}, pressure: [{:.3} , {:.3}], div: [{:.3} , {:.3}], vel: [{:.3} , {:.3}]",
            step,
            grid.stats[0].pressure,
            grid.stats[1].pressure,
            grid.stats[0].div,
            grid.stats[1].div,
            grid.stats[0].velocity_norm,
            grid.stats[1].velocity_norm
        ))
    } else {
        None
    };

    let mut file = params.output.replace("{}", &format!("smoke-{:06}", step));

    let smoke_color: &dyn plotting::ColorFunction = &|idx: Index2| {
        let alpha = grid.cell(idx).smoke.back;
        let mut color = cg.at(0.6 * alpha);
        color.a = alpha;
        return color;
    };

    plotting::grid(
        params.size,
        grid.dim,
        make_solid(&grid, &solid_color, &smoke_color),
        file,
        text.as_deref(),
    )?;

    if params.with_velocity {
        file = params.output.replace("{}", &format!("vel-{:06}", step));
        let cg: colorgrad::Gradient = colorgrad::turbo();

        let v_range = grid.stats[1].velocity_norm - grid.stats[0].velocity_norm;

        let get_color = |idx: Index2| {
            let t = grid.cell(idx).velocity.back.norm() - grid.stats[0].velocity_norm / v_range;
            return cg.at(t);
        };

        let get_color_masked = make_masked(&grid, &get_color);

        let velocity_color: &dyn ColorFunction = if params.with_velocity_masked {
            &get_color_masked
        } else {
            &get_color
        };

        plotting::grid(
            params.size,
            grid.dim,
            make_solid(&grid, &solid_color, &velocity_color),
            file,
            text.as_deref(),
        )?;
    }

    if params.with_pressure {
        let cg: colorgrad::Gradient = colorgrad::turbo();

        let p_range = grid.stats[1].pressure - grid.stats[0].pressure;

        let get_color: &dyn ColorFunction = &|idx: Index2| {
            let t = (grid.cell(idx).pressure - grid.stats[0].pressure) / p_range;
            return cg.at(t);
        };

        let get_color_masked = make_masked(&grid, &get_color);

        let pressure_color: &dyn plotting::ColorFunction = if params.with_velocity_masked {
            &get_color_masked
        } else {
            &get_color
        };

        file = params.output.replace("{}", &format!("press-{:06}", step));
        plotting::grid(
            params.size,
            grid.dim,
            make_solid(&grid, &solid_color, &pressure_color),
            file,
            text.as_deref(),
        )?;
    }

    return Ok(());
}
