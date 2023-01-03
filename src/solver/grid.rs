use crate::log::{debug, warn, Logger};
use crate::solver::timestepper::Integrate;
use crate::types::*;
use std::any::Any;
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
    pub dim: Index2,

    cells: Vec<Cell>,

    extent: Vector2,

    // Grid offsets for each axis of the velocity in the cells..
    offsets: [Vector2; 2],
}

type GridIndex = Index2;

pub struct GridIndexIterator {
    curr: GridIndex,

    min: Index2,
    max: Index2,
}

impl GridIndexIterator {
    pub fn new(dim: Index2) -> GridIndexIterator {
        return GridIndexIterator {
            curr: idx!(0, 0),
            min: idx!(0, 0),
            max: dim,
        };
    }
}

impl Iterator for GridIndexIterator {
    type Item = GridIndex;

    fn next(&mut self) -> Option<Self::Item> {
        let curr = self.curr; // Copy current.

        // Advance to next cell.
        let next = &mut self.curr;
        next.x += 1;
        if next.x >= self.max.x {
            next.y += 1;
            next.x = self.min.x;
        }

        if Grid::is_inside_range(self.min, self.max, curr) {
            return Some(curr);
        }

        return None;
    }
}

impl Grid {
    pub fn new(mut dim: Index2, cell_width: Scalar) -> Self {
        dim.x += 2;
        dim.y += 2;

        let h_2 = cell_width as Scalar * 0.5;
        let extent = dim.cast::<Scalar>() * cell_width;

        return Grid {
            dim,
            cell_width,

            cells: GridIndexIterator::new(dim)
                .map(|it| Cell::new(it))
                .collect(),

            extent,

            // `x`-values lie at offset `(0, h/2)` and
            // `y`-values at `(h/2, 0)`.
            offsets: [vec2!(0.0, h_2), vec2!(h_2, 0.0)],
        };
    }

    pub fn iter_index(&self) -> GridIndexIterator {
        return GridIndexIterator::new(self.dim);
    }

    pub fn iter_index_inside(&self) -> GridIndexIterator {
        return GridIndexIterator {
            curr: idx!(1, 1),
            min: idx!(1, 1),
            max: self.dim - idx!(1, 1),
        };
    }

    pub fn clamp_to_range<T>(min: Vector2T<T>, max: Vector2T<T>, index: Vector2T<T>) -> Vector2T<T>
    where
        T: nalgebra::Scalar + PartialOrd + Copy,
    {
        return Vector2T::<T>::new(
            nalgebra::clamp(index.x, min.x, max.x),
            nalgebra::clamp(index.y, min.y, max.y),
        );
    }

    pub fn is_inside_range(min: Index2, max: Index2, index: Index2) -> bool {
        return index < max && index >= min;
    }

    pub fn is_inside_border(&self, index: Index2) -> bool {
        return Grid::is_inside_range(Index2::zeros() + idx!(1, 1), self.dim - idx!(1, 1), index);
    }

    pub fn get_neighbors_indices(index: Index2) -> [[Index2; 2]; 2] {
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

    pub fn set_obstacle(&mut self, pos: Vector2, radius: f64) {
        for idx in self.iter_index_inside() {
            let c = idx.cast::<Scalar>() * self.cell_width
                + vec2!(self.cell_width * 0.5, self.cell_width * 0.5);

            if (c - pos).norm_squared() <= radius * radius {
                self.cell_mut(idx).mode = CellTypes::Solid;
            } else {
                self.cell_mut(idx).mode = CellTypes::Fluid;
            }
        }
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

    fn cell(&'t self, index: Index2) -> &Cell {
        return &self.cells[index.x + index.y * self.dim.x];
    }

    fn cell_mut(&'t mut self, index: Index2) -> &mut Cell {
        return &mut self.cells[index.x + index.y * self.dim.x];
    }

    fn cell_opt(&'t self, index: Index2) -> Option<&Cell> {
        return Grid::is_inside_range(Index2::zeros(), self.dim, index).then(|| self.cell(index));
    }

    fn cell_mut_opt(&'t mut self, index: Index2) -> Option<&mut Cell> {
        return Grid::is_inside_range(Index2::zeros(), self.dim, index)
            .then(|| self.cell_mut(index));
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
        self.velocity.front = match self.mode {
            CellTypes::Solid => self.velocity.back,
            CellTypes::Fluid => self.velocity.back + dt * gravity,
        };
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl Integrate for Grid {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn integrate(&mut self, log: &Logger, dt: Scalar, gravity: Vector2) {
        debug!(log, "Integrate grid.");

        for cell in self.cells.iter_mut() {
            cell.integrate(log, dt, gravity); // integrate
        }
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
            for it in self.iter_index_inside() {
                let index = it;

                assert!(
                    self.is_inside_border(index),
                    "Index {} is not inside",
                    index
                );

                if self.cell(index).mode == CellTypes::Solid {
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
                    nbs_s[dir] = vec2!(s_factor(nbs[dir][0]), s_factor(nbs[dir][1]));
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

        for it in self.iter_index() {
            self.cell_mut(it).velocity.swap();
        }
    }
}

impl Grid {
    pub fn sample_field<F: Fn(&Cell, usize) -> Scalar>(
        &self,
        log: &Logger,
        mut pos: Vector2,
        dir: usize,
        get_val: F,
    ) -> Scalar {
        let h = self.cell_width;
        let h_inv = 1.0 / self.cell_width;

        let offset = self.offsets[dir];
        pos = pos - offset; // Compute position on staggered grid.
        pos = Grid::clamp_to_range(Vector2::zeros(), self.extent, pos);

        // Compute index.
        let mut index = Index2::from_iterator((pos * h_inv).iter().map(|v| *v as usize));

        let clamp_index = |i| Grid::clamp_to_range(Index2::zeros(), self.dim, i);

        index = clamp_index(index);
        let pos_cell = pos - index.cast::<Scalar>() * h;
        let alpha = pos_cell * h_inv;

        // Get all neighbor indices. (column major).
        // [ (0,1), (1,1)
        //   (0,0), (1,0) ]
        let nbs = [
            clamp_index(index + Index2::new(0, 1)),
            index,
            clamp_index(index + Index2::new(1, 1)),
            clamp_index(index + Index2::new(1, 0)),
        ];

        // Get all values on the grid.
        let m = Matrix2::from_iterator(
            nbs.map(|i| {
                return get_val(self.cell(i), dir);
            })
            .into_iter(), // Column major order.
        );

        let t1 = vec2!(1.0 - alpha.x, alpha.x);
        let t2 = vec2!(alpha.y, 1.0 - alpha.y);
        debug!(log, "Sample: {} * {} * {}", t2.transpose(), m, t1);

        return t2.dot(&(m * t1));
    }
}
