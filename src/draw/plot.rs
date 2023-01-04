use crate::types::*;
use colorgrad;
use itertools::Itertools;
use plotters::prelude::*;
use plotters::style::Color;

use std::error::Error;

pub fn grid<F: Fn(Index2) -> Option<Scalar>>(
    size: Index2,
    dim: Index2,
    get_data: F,
    file: String,
    color_undef: Option<&dyn Color>,
    text: &str,
) -> Result<(), Box<dyn Error>> {
    let ratio = dim.y as Scalar / dim.x as Scalar;

    let size_px = dim!(size.x, (size.x as Scalar * ratio) as usize + 20);

    let root = BitMapBackend::new(&file, (size_px.x as u32, size_px.y as u32)).into_drawing_area();
    root.fill(&WHITE)?;
    root.titled(&text, ("sans-serif", 12))?;

    let cg: colorgrad::Gradient = colorgrad::turbo();

    let mut chart = ChartBuilder::on(&root)
        .margin_top(20)
        .x_label_area_size(0)
        .y_label_area_size(0)
        .build_cartesian_2d(0.0..(dim.x) as Scalar, 0.0..(dim.y) as Scalar)?;

    chart
        .configure_mesh()
        .disable_x_mesh()
        .disable_y_mesh()
        .axis_desc_style(("sans-serif", 15))
        .draw()?;

    let plotting_area = chart.plotting_area();

    let mut some_color: RGBColor;
    let none_color = color_undef.unwrap_or(&RED);
    let mut color: &dyn Color;

    for (i, j) in (0..dim.x).cartesian_product(0..dim.y) {
        match get_data(idx!(i, j)) {
            Some(v) => {
                let c = cg.at(v.clamp(0.0, 1.0)).to_rgba8();
                some_color = RGBColor(c[0], c[1], c[2]);
                color = &some_color;
            }
            None => color = none_color,
        };

        let x = i as Scalar;
        let y = j as Scalar;

        plotting_area.draw(&Rectangle::new(
            [(x, y), (x + 1.0, y + 1.0)],
            ShapeStyle::from(color.to_rgba()).filled(),
        ))?;
    }

    // To avoid the IO failure being ignored silently, we manually call the present function
    root.present().expect(
        "Unable to write result to file, please \
         make sure 'plotters-doc-data' dir exists under current dir",
    );

    return Ok(());
}
