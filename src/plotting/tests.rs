#[cfg(test)]
mod tests {

    use crate::plotting::plot::grid;
    use crate::types::*;
    use colorgrad;

    #[test]
    fn test_grid() -> Result<(), Box<dyn std::error::Error>> {
        let cg: colorgrad::Gradient = colorgrad::turbo();

        let get_color = |index: Index2| {
            if index.y == 300 - 1 {
                return cg.at(0.0);
            }

            return cg.at(((index.x as Scalar) / 15.0).sin() * ((index.y as Scalar) / 10.0).cos());
        };

        let file = "test.png";
        grid(
            dim!(500, 500),
            dim!(300, 300),
            get_color,
            file.to_string(),
            None,
        )?;

        Ok(())
    }
}
