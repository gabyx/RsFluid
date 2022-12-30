use crate::log::{debug, warn, Logger};
use crate::solver::timestepper::Integrate;
use crate::types::*;
use std::num::Wrapping;

#[derive(Clone, Debug, PartialEq)]
pub enum CellTypes {
    Solid,
    Fluid,
}

#[derive(Clone, Debug)]
pub struct Cell {
    // Velocity x,y:
    // - v_x is at the location (h/2, 0),
    // - v_y is at the location (0, h/2),
    pub velocity: FrontBackBuffer<Vector2>,

    pub pressure: Scalar,
    pub smoke: FrontBackBuffer<Scalar>,

    pub mode: CellTypes,

    index: Index2,
}

impl Cell {
    pub fn new(index: Index2) -> Self {
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

    pub fn index(&self) -> Index2 {
        return self.index;
    }
}

pub struct Grid {
    pub cell_width: Scalar,
    pub dim: Dimension2,

    cells: Vec<Cell>,
}

#[derive(Copy, Clone, Debug)]
pub struct GridIndex {
    pub index: Index2,
    dim: Dimension2,
}

pub struct GridIndexIterator {
    curr: GridIndex,
}

impl GridIndex {
    fn to_data_index(&self) -> usize {
        return self.index.x + self.dim.x * self.index.y;
    }
}

impl Iterator for GridIndexIterator {
    type Item = GridIndex;

    fn next(&mut self) -> Option<Self::Item> {
        let curr = self.curr;
        let next = &mut self.curr;

        // Advance the ite
        next.index.x += 1;

        if next.index.x >= next.dim.x {
            next.index.y += 1;
            next.index.x = 0;
        }

        if Grid::is_inside(curr.dim, curr.index) {
            return Some(curr);
        }

        return None;
    }
}

impl Grid {
    pub fn new(mut dim_x: usize, mut dim_y: usize, cell_width: Scalar) -> Self {
        dim_x += 2;
        dim_y += 2;
        let n = dim_x * dim_y;

        let mut grid = Grid {
            cell_width,
            dim: Dimension2::new(dim_x, dim_y),
            cells: vec![Cell::new(Index2::new(0, 0)); n],
        };

        // Setup grid.
        for it in grid.to_index_iter() {
            let mode = if Grid::is_inside_border(grid.dim, it.index) {
                CellTypes::Fluid
            } else {
                CellTypes::Solid
            };

            let mut cell = Cell::new(it.index);
            cell.mode = mode;

            grid.cells[it.to_data_index()] = cell;
        }

        return grid;
    }

    fn to_index_iter(&self) -> GridIndexIterator {
        return GridIndexIterator {
            curr: GridIndex {
                index: Index2::new(0, 0),
                dim: self.dim,
            },
        };
    }

    pub fn is_inside(dim: Dimension2, index: Index2) -> bool {
        return index < dim;
    }

    fn is_inside_border(dim: Dimension2, index: Index2) -> bool {
        return index > Index2::zeros() && index < (dim - Index2::new(1, 1));
    }

    fn get_neighbors_indices(index: Index2) -> [[Index2; 2]; 2] {
        let decrement = |x| (Wrapping(x) - Wrapping(1usize)).0;

        return [
            [
                // Negative neighbors.
                Index2::new(decrement(index.x), index.y),
                Index2::new(index.x, decrement(index.y)),
            ],
            [
                // Positive neighbors.
                Index2::new(index.x + 1, index.y),
                Index2::new(index.x, index.y + 1),
            ],
        ];
    }
}

pub trait CellGetter<'a, I> {
    type Item: 'a;

    type Output = &'a Self::Item;
    type OutputMut = &'a mut Self::Item;

    fn cell(&'a self, index: I) -> Self::Output;
    fn cell_mut(&'a mut self, index: I) -> Self::OutputMut;

    type OutputOpt = Option<&'a Self::Item>;
    type OutputMutOpt = Option<&'a mut Self::Item>;

    fn cell_opt(&'a self, index: Index2) -> Self::OutputOpt;
    fn cell_mut_opt(&'a mut self, index: Index2) -> Self::OutputMutOpt;
}

impl<'t> CellGetter<'t, Index2> for Grid {
    type Item = Cell;

    fn cell(&'t self, index: Index2) -> Self::Output {
        return &self.cells[index.x + index.y * self.dim.x];
    }

    fn cell_mut(&'t mut self, index: Index2) -> Self::OutputMut {
        return &mut self.cells[index.x + index.y * self.dim.x];
    }

    fn cell_opt(&'t self, index: Index2) -> Self::OutputOpt {
        return Grid::is_inside(self.dim, index).then(|| self.cell(index));
    }

    fn cell_mut_opt(&'t mut self, index: Index2) -> Self::OutputMutOpt {
        return Grid::is_inside(self.dim, index).then(|| self.cell_mut(index));
    }
}

impl Grid {
    pub fn modify_cells<F, const N: usize>(&mut self, indices: [usize; N], mut f: F) -> ()
    where
        F: FnMut([&mut Cell; N]),
    {
        let refs = self.cells.get_many_mut(indices).expect("Wrong indices.");
        f(refs);
    }
}

impl Integrate for Cell {
    fn integrate(&mut self, _log: &Logger, dt: Scalar, gravity: Vector2) {
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

    fn solve_incompressibility(
        &mut self,
        log: &Logger,
        dt: Scalar,
        iterations: u64,
        density: Scalar,
    ) {
        let r = 1.9; // Overrelaxation factor.

        let cp = density * self.cell_width / dt;

        for _iter in 0..iterations {
            for it in self.to_index_iter() {
                let index = it.index;
                let dim = self.dim;

                if !Grid::is_inside_border(dim, index) || self.cell(index).mode == CellTypes::Solid {
                    continue;
                }

                let s_factor = |index: Index2| {
                    return if self.cell(index).mode == CellTypes::Solid {
                        0.0
                    } else {
                        1.0
                    };
                };

                let nbs = Grid::get_neighbors_indices(index);

                // Normalization values `s`
                // for negative/positive neighbors.
                // - 0: solid, 1: fluid.
                let mut nbs_s = [Vector2::zeros(), Vector2::zeros()];
                let mut s = 0.0;

                for dir in 0..2 {
                    nbs_s[dir] = Vector2::new(s_factor(nbs[dir][0]), s_factor(nbs[dir][1]));
                    s += nbs_s[dir].sum();
                }

                if s == 0.0 {
                    warn!(log, "Fluid in-face count is 0.0 for {:?}", index);
                    continue;
                }

                let get_vel = |index: Index2, dir: usize| {
                    return self.cell(index).velocity.front[dir];
                };

                let mut div: Scalar = 0.0; // Net outflow on this cell.
                let pos_idx = 1usize;
                let nbs_pos = &nbs[pos_idx];
                for xy in 0..2 {
                    div += get_vel(nbs_pos[xy], xy) - get_vel(index, xy)
                }

                // Normalize outflow to the cells we can control.
                let p = div / s;
                self.cell_mut(index).pressure -= cp * p;

                // Add outflow-part to inflows to reach net 0-outflow.
                self.cell_mut(index).velocity.front += r * nbs_s[0] * p;

                // Subtract outflow-part to outflows to reach net 0-outflow.
                self.cell_mut(nbs[pos_idx][0]).velocity.front.x -= r * nbs_s[pos_idx].x * p;
                self.cell_mut(nbs[pos_idx][1]).velocity.front.y -= r * nbs_s[pos_idx].y * p;
            }
        }
    }
}

impl Grid {
    fn enforce_solid_constraints(&mut self, log: &Logger) {
        debug!(log, "Enforce solid constraints on solid cells.");

        // Enforce solid constraint over all cells which are solid.
        for it in self.to_index_iter() {
            let index = it.index;

            {
                let cell = self.cell_mut(index);
                if cell.mode != CellTypes::Solid {
                    continue;
                }

                // Cell is solid, so constrain all involved staggered velocity.
                // to the last one and also for the neighbors in x and y direction.
                cell.velocity.front = cell.velocity.back;
            }

            for idx in 0..2usize {
                let mut nb_index = index;

                match idx {
                    0 => nb_index.x += 1, // x neighbor.
                    1 => nb_index.y += 1, // y neighbor.
                    _ => {}
                }

                let cell = self.cell_mut_opt(nb_index);
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
