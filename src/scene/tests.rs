#[cfg(test)]
mod tests {

    use crate::log::*;
    use crate::scene::grid::*;
    use crate::scene::cell::*;
    use crate::types::*;
    use float_cmp::approx_eq;

    #[test]
    fn check_grid_sample() {
        let (log, _) = create_logger();
        let mut grid = Grid::new(dim!(10, 10), 1.0);

        let sample_back_vel = |cell: &Cell| {
            let v = cell.velocity.back[1];
            debug!(log, "Val {}", v);
            return v;
        };

        //   | 0,1 | 1,1 |
        // 1 |- 3 -|- 4 -|
        //   | 0,0 | 1,0 |
        //   |- 1 -|- 2 -|
        //   0 ----1---->2

        grid.cell_mut(idx!(0, 0)).velocity.back = vec2!(-1.0, 1.0);
        grid.cell_mut(idx!(1, 0)).velocity.back = vec2!(-1.0, 2.0);
        grid.cell_mut(idx!(0, 1)).velocity.back = vec2!(-1.0, 3.0);
        grid.cell_mut(idx!(1, 1)).velocity.back = vec2!(-1.0, 4.0);

        let min = idx!(0, 0);
        let max = grid.dim;

        let eps = Scalar::EPSILON;
        let val = grid.sample_field(min, max, vec2!(1.0, 1.0 - eps), Some(1), sample_back_vel);
        assert!(approx_eq!(Scalar, val, 3.5, ulps = 10), "Val: {}", val);

        let val = grid.sample_field(
            min,
            max,
            vec2!(1.5 - eps, 1.0 - eps),
            Some(1),
            sample_back_vel,
        );
        assert!(approx_eq!(Scalar, val, 4.0, ulps = 10), "Val: {}", val);

        let val = grid.sample_field(min, max, vec2!(1.0, 0.5), Some(1), sample_back_vel);
        assert!(approx_eq!(Scalar, val, 2.5, ulps = 10), "Val: {}", val);

        // Out of defined values field.
        let val = grid.sample_field(
            min,
            max,
            vec2!(2.5 - 2.0 * eps, 1.0 - eps),
            Some(1),
            sample_back_vel,
        );
        assert!(approx_eq!(Scalar, val, 0.0, epsilon = 1e-6), "Val: {}", val);
    }

}
