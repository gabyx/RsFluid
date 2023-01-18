use criterion::{
    criterion_group, criterion_main, AxisScale, BenchmarkId, Criterion, PlotConfiguration,
};
use itertools::Itertools;
use rayon::prelude::*;
use std::time::Duration;
type Index2 = nalgebra::Vector2<usize>;

#[macro_export]
macro_rules! idx {
    ($x:expr, $($y:expr),+ ) => {
        Index2::new($x, $($y),+)
    };
}

#[derive(Clone, Debug)]
pub struct C {
    pub i: usize,
    pub index: Index2,
}

#[derive(Clone, Debug)]
pub struct G {
    pub cells: Vec<C>,
    pub dim: Index2,
}

pub struct PosStencilMut<'a, T> {
    cell: &'a mut T,
    neighbors: [Option<&'a mut T>; 2],
}

impl<'a, T> PosStencilMut<'a, T> {
    #[inline(always)]
    fn get_neighbors<'cell>(cell: *mut T, dim: Index2, index: Index2) -> [Option<&'cell mut T>; 2] {
        assert!(dim.x >= 1 && dim.y >= 1);

        let mut nbs = [None, None];

        for dir in 0..2 {
            if index[dir] >= dim[dir] - 1 {
                // No neighbor possible.
                continue;
            }

            // For x-direction : offset = 1, for y-direction: offset = dim[0],
            // general: for n-direction: offset = dim[0]*dim[1]*...*dim[n-1]
            let offset = dim.iter().take(dir).fold(1, std::ops::Mul::mul);
            nbs[dir] = Some(unsafe { &mut *cell.add(offset) });
        }

        return nbs;
    }
}

pub trait PosStencil<'a, T: 'a> {
    fn positive_stencils_mut(
        &'a mut self,
        dim: Index2,
        min: Option<Index2>,
        max: Option<Index2>,
    ) -> Box<dyn Iterator<Item = PosStencilMut<'a, T>> + '_>;
}

impl<'a, T: 'a> PosStencil<'a, T> for Vec<T> {
    fn positive_stencils_mut(
        &'a mut self,
        dim: Index2,
        min: Option<Index2>,
        max: Option<Index2>,
    ) -> Box<dyn Iterator<Item = PosStencilMut<'a, T>> + '_> {
        return Box::new(positive_stencils_mut(self.as_mut_slice(), dim, min, max));
    }
}

fn positive_stencils_mut<T>(
    data: &mut [T],
    dim: Index2,
    min: Option<Index2>,
    max: Option<Index2>,
) -> impl Iterator<Item = PosStencilMut<'_, T>> {
    assert!(
        dim > idx!(0, 0) && dim.iter().fold(1, std::ops::Mul::mul) == data.len(),
        "Wrong dimensions."
    );

    let min = min.unwrap_or(Index2::zeros());
    let max = max.unwrap_or(dim - idx!(1, 1));

    assert!(min >= Index2::zeros() && max < dim);

    return (min[0]..max[0])
        .step_by(2)
        .cartesian_product((min[1]..max[1]).step_by(2))
        .map(move |(i, j)| {
            let index = idx!(i, j);
            let cell: *mut T = &mut data[index[0] + index[1] * dim[0]];

            // Here the unsafe part happens.
            // Get two non-aliasing mutable references for the neighbors.
            return PosStencilMut {
                cell: unsafe { &mut *cell },
                neighbors: PosStencilMut::get_neighbors::<'_>(cell, dim, index),
            };
        });
}

fn test() {
    let mut v = vec![1, 2, 3, 4];
    let s = positive_stencils_mut(v.as_mut_slice(), idx!(2, 2), None, None)
        .next()
        .unwrap();
    // drop(v); // This should invalidate the life-time of `s` but it does not???
    *s.cell += 3;
}

// Safety: `PosStencilMut` can only be created by `pos_stencils_mut`
// which guarantees non-aliased mutable references.
// Therefore it can safely be transferred to another thread.
unsafe impl<'a, T> Send for PosStencilMut<'a, T> {}

// For PosStencilMut to be Sync we have to enforce that you can't write to something stored
// in a &PosStencilMut while that same something could be read or written to from another &PosStencilMut.
// Since you need an &mut PosStencilMut to write to the pointer, and the borrow checker enforces that
// mutable references must be exclusive, there are no soundness issues making PosStencilMut sync either.
unsafe impl<'a, T> Sync for PosStencilMut<'a, T> {}

#[inline(always)]
fn stencil_compute(stencil: &mut PosStencilMut<'_, C>) {
    stencil.cell.i += match stencil.neighbors[0].as_deref() {
        Some(c) => c.i,
        None => 0,
    } + match stencil.neighbors[1].as_deref() {
        Some(c) => c.i,
        None => 0,
    };

    if let Some(n) = stencil.neighbors[0].as_deref_mut() {
        n.i += stencil.cell.i;
    }
}

fn run_grid_parallel(n: usize, grid: &mut G) {
    let mut stencils: Vec<_> =
        positive_stencils_mut(grid.cells.as_mut_slice(), grid.dim, None, None).collect();

    for _ in 0..n {
        stencils
            .par_iter_mut()
            .for_each(|stencil| stencil_compute(stencil))
    }
}

fn run_grid_single(n: usize, grid: &mut G) {
    let mut stencils: Vec<_> = grid
        .cells
        .positive_stencils_mut(grid.dim, None, None)
        .collect();

    for _ in 0..n {
        stencils
            .iter_mut()
            .for_each(|stencil| stencil_compute(stencil))
    }
}

fn criterion_benchmark(c: &mut Criterion) {
    let mut dim = idx!(100, 100);

    let mut dim2 = idx!(100, 100);

    let i = 4;
    let a =  &dim;
    let b: &mut Index2 = &mut (*a);
    b.x =3 ;

    let cell_generator = (0..dim[0]).cartesian_product(0..dim[1]).map(|(i, j)| C {
        i: i + j,
        index: idx!(i, j),
    });

    let mut grid = G {
        cells: Vec::from_iter(cell_generator),
        dim,
    };

    test();

    let mut group = c.benchmark_group("Grid Single vs. Parallel");
    for i in [10, 40, 100, 500, 1000].iter() {
        group.bench_with_input(BenchmarkId::new("Single", i), i, |b, i| {
            b.iter(|| run_grid_parallel(*i, &mut grid))
        });

        group.bench_with_input(BenchmarkId::new("Parallel", i), i, |b, i| {
            b.iter(|| run_grid_single(*i, &mut grid))
        });
    }

    let plot_config = PlotConfiguration::default().summary_scale(AxisScale::Logarithmic);
    group.plot_config(plot_config);

    group.measurement_time(Duration::from_secs(20));
    group.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
