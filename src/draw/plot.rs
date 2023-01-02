use crate::types::*;
use colorgrad;
use itertools::Itertools;
use plotters::prelude::*;
use std::error::Error;

pub fn grid<F: Fn(usize, usize) -> Scalar>(
    size: Index2,
    dim: Index2,
    get_data: F,
    file: String,
) -> Result<(), Box<dyn Error>> {

    let ratio = dim.y as Scalar / dim.x as Scalar;
    let size_px = dim!(size.x, (size.y as Scalar * ratio) as usize);

    let root = BitMapBackend::new(&file, (size_px.x as u32, size_px.y as u32)).into_drawing_area();

    root.fill(&WHITE)?;
    root.titled("Velocity", ("sans-serif", 12))?;

    let cg: colorgrad::Gradient = colorgrad::viridis();

    let mut chart = ChartBuilder::on(&root)
        .margin(40)
        .x_label_area_size(40)
        .y_label_area_size(40)
        .build_cartesian_2d(0.0..dim.x as Scalar, 0.0..dim.y as Scalar)?;

    chart
        .configure_mesh()
        .disable_x_mesh()
        .disable_y_mesh()
        .axis_desc_style(("sans-serif", 15))
        .draw()?;

    let plotting_area = chart.plotting_area();

    for (i, j) in (0..dim.x).cartesian_product(0..dim.y) {
        let value = get_data(i, j);

        let x = i as Scalar;
        let y = j as Scalar;

        let color = cg.at(value.clamp(0.0, 1.0)).to_rgba8();

        plotting_area.draw(&Rectangle::new(
            [(x, y), (x + 1.0, y + 1.0)],
            ShapeStyle::from(&RGBColor(color[0], color[1], color[2])).filled(),
        ))?;
    }

    // To avoid the IO failure being ignored silently, we manually call the present function
    root.present().expect(
        "Unable to write result to file, please \
         make sure 'plotters-doc-data' dir exists under current dir",
    );

    return Ok(());
}
