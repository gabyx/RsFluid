#[cfg(test)]
mod tests {

    use crate::draw::plot::grid;
    use crate::types::*;

    #[test]
    fn test_grid() -> Result<(), Box<dyn std::error::Error>> {
        let get_data = |i, j| {
            return ((i as Scalar)/15.0).sin() * ((j as Scalar)/10.0).cos();
        };

        let file = "test.png";
        grid(
            dim!(800, 600),
            dim!(800, 600),
            get_data,
            file.to_string(),
        )?;

        Ok(())
    }
}
