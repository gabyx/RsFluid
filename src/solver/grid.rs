use slog::{debug, info, Logger};

use crate::solver::timestepper::Integrate;
use crate::types::*;

#[derive(Clone, Debug, PartialEq)]
pub enum CellTypes {
    Solid,
    Fluid,
}

#[derive(Clone, Debug, Copy)]
pub struct GlobalIndex(usize, usize);
pub struct InsideIndex(usize, usize);

#[derive(Clone, Debug)]
pub struct Cell {
    // Velocity x,y:
    // - v_x is at the location (h/2, 0),
    // - v_y is at the location (0, h/2),
    pub velocity: FrontBackBuffer<Vector2>,

    pub pressure: Scalar,
    pub smoke: FrontBackBuffer<Scalar>,

    pub mode: CellTypes,

    index: GlobalIndex,
}

impl Cell {
    pub fn new(index: GlobalIndex) -> Self {
        let default_vel = Vector2::from_element(0.0);
        let default_pressure = 0.0;
        let default_smoke = 0.0;

        return Cell {
            velocity: FrontBackBuffer {
                front: default_vel,
                back: default_vel,
            },
            pressure: default_pressure,
            smoke: FrontBackBuffer {
                front: default_smoke,
                back: default_smoke,
            },
            mode: CellTypes::Fluid,
            index,
        };
    }

    pub fn index(&self) -> GlobalIndex {
        return self.index;
    }
}

pub struct Grid {
    pub cell_width: Scalar,
    pub dim_x: usize,
    pub dim_y: usize,

    cells: Vec<Cell>,
}

impl Grid {
    pub fn new(mut dim_x: usize, mut dim_y: usize, cell_width: Scalar) -> Self {
        dim_x += 2;
        dim_y += 2;
        let n = dim_x * dim_y;

        let mut grid = Grid {
            cell_width,
            dim_x,
            dim_y,
            cells: vec![Cell::new(GlobalIndex(0, 0)); n],
        };

        // Setup grid.
        for i in 0..dim_x {
            for j in 0..dim_y {
                let mode = if grid.is_border(GlobalIndex(i, j)) {
                    CellTypes::Solid
                } else {
                    CellTypes::Fluid
                };

                let mut cell = Cell::new(GlobalIndex(i, j));
                cell.mode = mode;

                grid.cells[i + j * dim_x] = cell;
            }
        }

        return grid;
    }

    fn is_border(&self, index: GlobalIndex) -> bool {
        return index.0 == 0
            || index.1 == 0
            || index.0 == self.dim_x - 1
            || index.1 == self.dim_y - 1;
    }
}

trait CellGetter<Index> {
    type Output;
    fn cell(self, index: Index) -> Self::Output;
}

// CellGetter::Self == &Grid
impl<'a> CellGetter<GlobalIndex> for &'a Grid {
    type Output = Option<&'a Cell>;

    fn cell(self, index: GlobalIndex) -> Self::Output {
        if index.0 < self.dim_x && index.1 < self.dim_y {
            return Some(&self.cells[index.0 + index.1 * self.dim_x]);
        }
        return None;
    }
}

// CellGetter::Self == &mut Grid
impl<'a> CellGetter<GlobalIndex> for &'a mut Grid {
    type Output = Option<&'a mut Cell>;

    fn cell(self, index: GlobalIndex) -> Self::Output {
        if index.0 < self.dim_x && index.1 < self.dim_y {
            return Some(&mut self.cells[index.0 + index.1 * self.dim_x]);
        }
        return None;
    }
}

// CellGetter::Self == &Grid
impl<'a> CellGetter<InsideIndex> for &'a Grid {
    type Output = Option<&'a Cell>;

    fn cell(self, index: InsideIndex) -> Self::Output {
        let index = GlobalIndex(index.0 + 1, index.1 + 1);
        return self.cell(index);
    }
}

// CellGetter::Self == &mut Grid
impl<'a> CellGetter<InsideIndex> for &'a mut Grid {
    type Output = Option<&'a mut Cell>;

    fn cell(self, index: InsideIndex) -> Self::Output {
        let index = GlobalIndex(index.0 + 1, index.1 + 1);
        return self.cell(index);
    }
}

impl Integrate for Cell {
    fn integrate(&mut self, log: &Logger, dt: Scalar, gravity: Vector2) {
        self.velocity.front = self.velocity.back + dt * gravity;
    }
}

impl Integrate for Grid {
    fn integrate(&mut self, log: &Logger, dt: Scalar, gravity: Vector2) {
        debug!(log, "Integrate grid.");

        for cell in self.cells.iter_mut() {
            cell.integrate(log, dt, gravity); // integrate
        }

        self.enforce_solid_constraints(log);
    }
}

impl Grid {
    fn enforce_solid_constraints(&mut self, log: &Logger) {
        debug!(log, "Enforce solid constraints on solid cells.");

        // Enforce solid constraint over all cells which are solid.
        for i in 0..self.cells.len() {
            let index: GlobalIndex;
            {
                let mut cell = &mut self.cells[i];
                if cell.mode != CellTypes::Solid {
                    continue;
                }

                // Cell is solid, so constrain all involved staggered velocity.
                // to the last one.
                cell.velocity.front = cell.velocity.back;
                index = cell.index;
            }

            for idx in 0..2usize {
                let mut nb_index = index;

                match idx {
                    0 => nb_index.0 += 1, // x neighbor.
                    1 => nb_index.1 += 1, // y neighbor.
                    _ => {}
                }

                let cell = self.cell(nb_index);
                match cell {
                    Some(c) => {
                        c.velocity.front[idx] = c.velocity.back[idx]; // reset only the x,y direction.
                    }
                    None => {}
                }
            }
        }
    }
}
