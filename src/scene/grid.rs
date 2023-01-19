use crate::log::{debug, info, warn, Logger};
use crate::scene::cell::*;
use crate::scene::cell_stats::*;
use crate::scene::grid_stencil::*;
use crate::scene::timestepper::Integrate;
use crate::types::*;
use itertools::Itertools;
use rayon::prelude::*;
use std::any::Any;
use std::num::Wrapping;

pub struct Grid {
    pub cell_width: Scalar,
    pub dim: Index2,

    pub stats: [Stats; 2], //Min and max. accumulator statistics.

    cells: Vec<Cell>,

    extent: Vector2,

    // Grid offsets for each axis of the velocity in the cells..
    offsets: [Vector2; 2],
}

#[derive(Clone)]
pub struct GridIndexIterator {
    curr: Index2,

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
    type Item = Index2;

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

            stats: [Stats::min_identity(), Stats::max_identity()],

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

    pub fn set_obstacle(&mut self, pos: Vector2, radius: f64, velocity: Option<Vector2>) {
        let vel = velocity.unwrap_or(Vector2::zeros());

        for idx in self.iter_index_inside() {
            let c = (idx.cast::<Scalar>() + vec2!(0.5, 0.5)) * self.cell_width;

            if (c - pos).norm_squared() <= radius * radius {
                let c = self.cell_mut(idx);
                c.mode = CellTypes::Solid;
                c.velocity.back = vel;
            } else {
                self.cell_mut(idx).mode = CellTypes::Fluid;
            }
        }
    }

    fn compute_stats(&mut self, log: &Logger) {
        // Parallelized accumulation of statistics.
        self.stats[0] = self
            .cells
            .par_iter()
            .map(|c| Stats::from(c))
            .reduce(|| Stats::identity::<0>(), |a, b| Stats::min(&a, &b));

        self.stats[1] = self
            .cells
            .par_iter()
            .map(|c| Stats::from(c))
            .reduce(|| Stats::identity::<1>(), |a, b| Stats::max(&a, &b));

        info!(
            log,
            "Divergence range: {:.4?}, {:.4?}", self.stats[0].div, self.stats[1].div
        );
        info!(
            log,
            "Pressure range: {:.4?}, {:.4?}", self.stats[0].pressure, self.stats[1].pressure
        );
        info!(
            log,
            "Velocity range: {:.4?}, {:.4?}",
            self.stats[0].velocity_norm,
            self.stats[1].velocity_norm
        );
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

    fn cell(&self, index: Index2) -> &Cell {
        return &self.cells[index.x + index.y * self.dim.x];
    }

    fn cell_mut(&mut self, index: Index2) -> &mut Cell {
        return &mut self.cells[index.x + index.y * self.dim.x];
    }

    fn cell_opt(&self, index: Index2) -> Option<&Cell> {
        return Grid::is_inside_range(Index2::zeros(), self.dim, index).then(|| self.cell(index));
    }

    fn cell_mut_opt(&mut self, index: Index2) -> Option<&mut Cell> {
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

impl Integrate for Grid {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn reset(&mut self, log: &Logger) {
        info!(log, "Reset stats.");
        self.stats = [Stats::min_identity(), Stats::max_identity()];
    }

    fn integrate(&mut self, log: &Logger, dt: Scalar, gravity: Vector2) {
        debug!(log, "Integrate grid.");

        for cell in self.cells.iter_mut() {
            cell.integrate(log, dt, gravity); // integrate
        }

        // Extrapolate to fluid cells on border.
        let ranges = [
            [idx!(0, 1), idx!(0, self.dim.y)],
            [idx!(self.dim.x - 1, self.dim.x), idx!(0, self.dim.y)],
            [idx!(0, self.dim.x), idx!(0, 1)],
            [idx!(0, self.dim.x), idx!(self.dim.y - 1, self.dim.y)],
        ];

        debug!(log, "Extrapolate border.");

        for range in ranges {
            let xr = range[0];
            let yr = range[1];

            for (i, j) in (xr[0]..xr[1]).cartesian_product(yr[0]..yr[1]) {
                let idx = idx!(i, j);

                if self.cell(idx).mode == CellTypes::Solid {
                    continue;
                }

                for dir in 0..2 {
                    let pos = idx.cast::<Scalar>() * self.cell_width + self.offsets[dir];

                    // Just sample on the inside grid by clamping.
                    self.cell_mut(idx).velocity.back[dir] = self.sample_field(
                        idx!(1, 1),
                        self.dim - idx!(1, 1),
                        pos,
                        Some(dir),
                        |cell: &Cell| cell.velocity.back[dir],
                    );
                }
            }
        }
    }

    fn solve_incompressibility(
        &mut self,
        log: &Logger,
        dt: Scalar,
        iterations: u64,
        density: Scalar,
        parallel: bool,
    ) {
        if parallel {
            self.solve_incompressibility_parallel(log, dt, iterations, density);
        } else {
            self.solve_incompressibility_sequential(log, dt, iterations, density);
        }
    }

    fn advect(&mut self, log: &slog::Logger, dt: Scalar) {
        self.advect_velocity(log, dt);
        self.advect_smoke(log, dt);
    }
}

impl Grid {
    fn apply_pos_stencils<T>(&mut self, func: T)
    where
        T: Fn(&mut PosStencilMut<Cell>) + Send + Sync,
    {
        const OFFSETS: [Index2; 4] = [idx!(0, 0), idx!(1, 0), idx!(0, 1), idx!(1, 1)];

        for offset in OFFSETS.iter() {
            let mut stencils: Vec<_> = positive_stencils_mut(
                self.cells.as_mut_slice(),
                self.dim,
                Some(idx!(1, 1)),
                Some(self.dim - idx!(1, 1)),
                Some(*offset),
            )
            .collect();

            stencils.par_iter_mut().for_each(&func);
        }
    }

    fn solve_incompressibility_parallel(
        &mut self,
        log: &Logger,
        dt: Scalar,
        iterations: u64,
        density: Scalar,
    ) {
        assert!(
            (self.dim.x - 1) % 2 == 0 && (self.dim.y - 1) % 2 == 0,
            "Internal grid dimensions (dim = {} - 1) must be divisible
             by 2 in each direction.",
            self.dim
        );

        let r = 1.9; // Overrelaxation factor.
        let cp = density * self.cell_width / dt;

        let s_factor = |cell: &mut Cell| {
            return if cell.mode == CellTypes::Solid {
                0.0
            } else {
                1.0
            };
        };

        debug!(log, "Distribute all 's' factors for total sum.");
        self.apply_pos_stencils(|s: &mut PosStencilMut<Cell>| {
            // This cell (1: pos, 0: x)  <-- s from pos x-neighbor.
            s.cell.s_nbs[1][0] = s_factor(s.neighbors[0]);
            // Pos. x-neighbor (0: neg, 0: x) <-- s from this cell.
            s.neighbors[0].s_nbs[0][0] = s_factor(s.cell);

            // This cell (1: pos, 1: y) <-- s from pos y-neighbor.
            s.cell.s_nbs[1][1] = s_factor(s.neighbors[1]);
            // Pos. x-neighbor (0: neg, 1: y) <-- s from this cell.
            s.neighbors[1].s_nbs[0][1] = s_factor(s.cell);
        });

        debug!(log, "Sum all 's' factors in all cells.");
        self.cells.par_iter_mut().for_each(|c: &mut Cell| {
            // Reset pressure field.
            c.pressure = 0.0;

            let mut sum = 0.0;
            c.s_nbs.iter().for_each(|s| {
                sum += s.sum();
            });

            // Store the inverse.
            c.s_tot_inv = if sum != 0.0 {
                1.0 / sum
            } else {
                warn!(
                    log,
                    "Cell with index: '{}' contains only fluid neighbors.", c.index()
                );
                0.0
            };
        });

        for _iter in 0..iterations {
            self.apply_pos_stencils(|s: &mut PosStencilMut<Cell>| {
                if s.cell.s_tot_inv == 0.0 {
                    debug!(
                        log,
                        "Cell with index: '{}' contains only fluid neighbors.", s.cell.index()
                    );
                    return;
                }

                s.cell.div = 0.0;
                for dir in 0..2 {
                    s.cell.div += s.neighbors[dir].velocity.back[dir] - s.cell.velocity.back[dir]
                }
                s.cell.div_normed = s.cell.div * s.cell.s_tot_inv;

                s.cell.pressure -= cp * s.cell.div_normed;

                // Velocity update own cell.
                s.cell.velocity.back += r * cp * s.cell.s_nbs[0] * s.cell.div_normed;

                // Velocity update neighbors in x-direction.
                // Solid cells have s_nbs[_] == 0.
                s.neighbors[0].velocity.back[0] -= r * s.cell.s_nbs[1].x * s.cell.div_normed;

                // Velocity update neighbors in z-direction.
                // Solid cells have s_nbs[_] == 0.
                s.neighbors[1].velocity.back[1] -= r * s.cell.s_nbs[1].y * s.cell.div_normed;
            });
        }
    }

    fn solve_incompressibility_sequential(
        &mut self,
        log: &Logger,
        dt: Scalar,
        iterations: u64,
        density: Scalar,
    ) {
        // Set pressure field to zero.
        self.cells.par_iter_mut().for_each(|c| c.pressure = 0.0);

        let r = 1.9; // Overrelaxation factor.
        let cp = density * self.cell_width / dt;

        for _iter in 0..iterations {
            for idx in self.iter_index_inside() {
                if self.cell(idx).mode == CellTypes::Solid {
                    continue;
                }

                let s_factor = |index: Index2| {
                    return if self.cell(index).mode == CellTypes::Solid {
                        0.0
                    } else {
                        1.0
                    };
                };

                let nbs = Grid::get_neighbors_indices(idx);

                // Normalization values `s`
                // for negative/positive neighbors.
                // - 0: solid, 1: fluid.
                let mut s_nbs = [Vector2::zeros(), Vector2::zeros()];
                let mut s = 0.0;

                for neg_pos in 0..2 {
                    s_nbs[neg_pos] = vec2!(s_factor(nbs[neg_pos][0]), s_factor(nbs[neg_pos][1]));
                    s += s_nbs[neg_pos].sum();
                }

                if s == 0.0 {
                    warn!(log, "Fluid in-face count is 0.0 for {:?}", idx);
                    continue;
                }

                let get_vel = |index: Index2, dir: usize| {
                    return self.cell(index).velocity.back[dir];
                };

                let mut div: Scalar = 0.0; // Net outflow on this cell.
                let pos_idx = 1;
                let pos_nbs = &nbs[pos_idx];
                for dir in 0..2 {
                    div += get_vel(pos_nbs[dir], dir) - get_vel(idx, dir)
                }

                self.cell_mut(idx).div = div;

                // Normalize outflow to the cells we can control.
                let div_normed = div / s;
                self.cell_mut(idx).pressure -= cp * div_normed;

                // Add outflow-part to inflows to reach net 0-outflow.
                // Solid cells have s_nbs[0] == 0.
                self.cell_mut(idx).velocity.back += r * s_nbs[0] * div_normed;

                // Subtract outflow-part to outflows to iteratively reach net 0-outflow (div(v) == 0).
                // Solid cells have s_nbs[0] == 0.
                self.cell_mut(nbs[pos_idx][0]).velocity.back.x -=
                    r * (s_nbs[pos_idx].x) * div_normed;

                // Solid cells have s_nbs[0] == 0.
                self.cell_mut(nbs[pos_idx][1]).velocity.back.y -= r * s_nbs[pos_idx].y * div_normed;
            }
        }

        self.compute_stats(&log);
    }

    fn advect_velocity(&mut self, log: &slog::Logger, dt: Scalar) {
        debug!(log, "Advect velocity.");

        self.cells
            .par_iter_mut()
            .for_each(|c| c.velocity.front = c.velocity.back);

        for idx in self.iter_index_inside() {
            if self.cell(idx).mode == CellTypes::Solid {
                continue;
            }

            let nbs = Grid::get_neighbors_indices(idx);

            // Advect the two staggered grids (x and then y-direction).
            for dir in 0..2 {
                // Is the negative neighbor a solid cell, then do not advect this velocity.
                if self.cell(nbs[0][dir]).mode == CellTypes::Solid {
                    continue;
                }

                let mut pos = idx.cast::<Scalar>() * self.cell_width + self.offsets[dir];
                let mut vel: Vector2 = self.cell(idx).velocity.back;

                let sample = |pos: Vector2, dir: usize| {
                    return self.sample_field(
                        idx!(1, 1),
                        self.dim - idx!(1, 1),
                        pos,
                        Some(dir),
                        |cell: &Cell| cell.velocity.back[dir],
                    );
                };

                let other_dir = (dir + 1) % 2;
                vel[other_dir] = sample(pos, other_dir);

                // Get position of particle which reached this position.
                pos = pos - dt * vel;
                //debug!(log, "Idx: {}", idx);

                // Set the past velocity at this cell.
                self.cell_mut(idx).velocity.front[dir] = sample(pos, dir);
            }
        }

        self.cells.par_iter_mut().for_each(|c| c.velocity.swap());
    }

    fn advect_smoke(&mut self, log: &slog::Logger, dt: Scalar) {
        debug!(log, "Advect smoke.");

        self.cells
            .par_iter_mut()
            .for_each(|c| c.smoke.front = c.smoke.back);

        for idx in self.iter_index_inside() {
            if self.cell(idx).mode == CellTypes::Solid {
                continue;
            }

            let nbs = Grid::get_neighbors_indices(idx);
            let mut pos = (idx.cast::<Scalar>() + vec2!(0.5, 0.5)) * self.cell_width;

            let mut vel = Vector2::zeros();
            for dir in 0..2 {
                vel += vec2!(
                    self.cell(nbs[dir][0]).velocity.back.x,
                    self.cell(nbs[dir][1]).velocity.back.y
                ) * 0.5;
            }

            pos = pos - dt * vel;

            self.cell_mut(idx).smoke.front = self.sample_field(
                idx!(0, 0),
                self.dim - idx!(0, 0),
                pos,
                None,
                |cell: &Cell| cell.smoke.back,
            );
        }

        self.cells.par_iter_mut().for_each(|c| c.smoke.swap());
    }

    pub fn sample_field<F: Fn(&Cell) -> Scalar>(
        &self,
        min: Index2,
        max: Index2,
        mut pos: Vector2,
        dir: Option<usize>,
        get_val: F,
    ) -> Scalar {
        let h = self.cell_width;
        let h_inv = 1.0 / self.cell_width;

        // If `dir` is set, we need some offset.
        // For velocities as they are on a staggered grid.
        let offset = dir.map_or(Vector2::zeros(), |d| self.offsets[d]);
        pos = pos - offset; // Compute position on staggered grid.
        pos = Grid::clamp_to_range(Vector2::zeros(), self.extent, pos);

        // Compute index.
        let mut index = Index2::from_iterator((pos * h_inv).iter().map(|v| *v as usize));

        let clamp_index = |i| Grid::clamp_to_range(min, max - idx!(1, 1), i);

        index = clamp_index(index);
        let pos_cell = pos - index.cast::<Scalar>() * h;
        let alpha = Grid::clamp_to_range(vec2!(0.0, 0.0), vec2!(1.0, 1.0), pos_cell * h_inv);

        // debug!(log, "Sample at: {}", index);

        // Get all neighbor indices (column major).
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
                return get_val(self.cell(i));
            })
            .into_iter(), // Column major order.
        );

        let t1 = vec2!(1.0 - alpha.x, alpha.x);
        let t2 = vec2!(alpha.y, 1.0 - alpha.y);

        return t2.dot(&(m * t1));
    }
}
