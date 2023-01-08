use crate::types::*;
use colorgrad;
use itertools::Itertools;
use plotters::prelude::*;

use std::error::Error;

pub trait ColorFunction = Fn(Index2) -> colorgrad::Color;

pub fn grid<F: ColorFunction>(
    size: Index2,
    dim: Index2,
    get_color: F,
    file: String,
    text: Option<&str>,
) -> Result<(), Box<dyn Error>> {
    let ratio = dim.y as Scalar / dim.x as Scalar;

    let border_top = if text.is_some() { 25.0 } else { 0.0 };

    let size_px = dim!(size.x, (size.x as Scalar * ratio + border_top) as usize);

    let root = BitMapBackend::new(&file, (size_px.x as u32, size_px.y as u32)).into_drawing_area();
    root.fill(&BLACK)?;

    let text_style = ("sans-serif", 20)
        .with_color(WHITE)
        .into_text_style(&root);

    if let Some(text) = text {
        root.titled(&text, &text_style)?;
    }

    let mut chart = ChartBuilder::on(&root)
        .margin_top(border_top)
        .x_label_area_size(0)
        .y_label_area_size(0)
        .build_cartesian_2d(0.0..(dim.x) as Scalar, 0.0..(dim.y) as Scalar)?;

    chart
        .configure_mesh()
        .disable_x_mesh()
        .disable_y_mesh()
        .draw()?;

    let plotting_area = chart.plotting_area();

    let mut color: RGBAColor;

    // Iterate over all cells and get color.
    for (i, j) in (0..dim.x).cartesian_product(0..dim.y) {
        let c = get_color(idx!(i, j));

        color = RGBAColor(
            (c.r * 256.0) as u8,
            (c.g * 256.0) as u8,
            (c.b * 256.0) as u8,
            c.a,
        );

        let x = i as Scalar;
        let y = j as Scalar;

        plotting_area.draw(&Rectangle::new(
            [(x, y), (x + 1.0, y + 1.0)],
            ShapeStyle::from(color).filled(),
        ))?;
    }

    // To avoid the IO failure being ignored silently, we manually call the present function
    root.present().expect(
        "Unable to write result to file, please \
         make sure 'plotters-doc-data' dir exists under current dir",
    );

    return Ok(());
}
