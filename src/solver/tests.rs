#[cfg(test)]
mod tests {

    use crate::solver::grid::*;
    use crate::types::*;

    #[test]
    fn check_clamp() {
        let e = Index2::new(3, 8);
        let min = Index2::new(4, 5);
        let max = Index2::new(6, 7);
        let c = Grid::clamp_to_range(min, max, e);

        assert!(c == Index2::new(4, 7), "Clamp not working {}", c);
    }

    #[test]
    fn check_grid() {
        let mut grid = Grid::new(10, 10, 1.0);

        let sample_back_vel = |cell: &Cell, dir: usize| cell.velocity.back[dir];

        let val = grid.sample_field(Vector2::new(-1.0, -1.0), 0, sample_back_vel);

        assert!(val == 0.0);

      //   | 0,1 | 1,1 |
      // 1 |- 3 -|- 4 -|
      //   | 0,0 | 0,1 |
      //   |- 1 -|- 2 -|
      //   0 ----1---->2

        grid.cell_mut(idx!(0, 0)).velocity.back = Vector2::new(1.0, 1.0);
        grid.cell_mut(idx!(0, 1)).velocity.back = Vector2::new(2.0, 1.0);
        grid.cell_mut(idx!(1, 0)).velocity.back = Vector2::new(3.0, 1.0);
        grid.cell_mut(idx!(1, 1)).velocity.back = Vector2::new(4.0, 1.0);

        let val = grid.sample_field(Vector2::new(1.0, 1.0), 0, sample_back_vel);
        assert!(val != 0.0);
    }
}
