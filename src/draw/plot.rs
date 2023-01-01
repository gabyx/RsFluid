use crate::types::*;
use nalgebra as na;
use plotters::prelude::*;
use std::error::Error;
use std::iter::zip;

pub fn grid<F: Fn(usize, usize) -> Scalar>(
    dim: Index2,
    get_data: F,
    file: String,
) -> Result<(), Box<dyn Error>> {
    let ratio = dim.y as Scalar / dim.x as Scalar;
    let width = 800;
    let size_px = dim!(width, (width as Scalar * ratio) as usize);

    let root = BitMapBackend::new(&file, (size_px.x as u32, size_px.y as u32)).into_drawing_area();

    root.fill(&WHITE)?;

    let mut chart = ChartBuilder::on(&root)
        .margin(20)
        .x_label_area_size(10)
        .y_label_area_size(10)
        .build_cartesian_2d(0.0..dim.x as Scalar, 0.0..dim.y as Scalar)?;

    chart
        .configure_mesh()
        .disable_x_mesh()
        .disable_y_mesh()
        .draw()?;

    let plotting_area = chart.plotting_area();

    for (i, j) in zip(0..dim.x, 0..dim.y) {
        let value = get_data(i, j);
        let x = i as Scalar;
        let y = j as Scalar;

        let color = &HSLColor(
            240.0 / 360.0 - 240.0 / 360.0 * (value / 10.0) / 5.0,
            1.0,
            0.7,
        );

        plotting_area.draw(&Rectangle::new(
            [(x, y), (x + 1.0, y + 1.0)],
            ShapeStyle::from(&color).filled(),
        ))?;
    }

    // To avoid the IO failure being ignored silently, we manually call the present function
    root.present().expect(
        "Unable to write result to file, please \
         make sure 'plotters-doc-data' dir exists under current dir",
    );

    return Ok(());
}
