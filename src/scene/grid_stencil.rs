use crate::types::*;
use rayon::iter::{IndexedParallelIterator, ParallelIterator};
use rayon::slice::ParallelSliceMut;

pub struct PosStencilMut<'a, T>
where
    T: Send + Sync,
{
    /// The current cell.
    pub cell: &'a mut T,

    /// The positive neighbors in `x`,`y`-direction.
    pub neighbors: [&'a mut T; 2],
}

/// First dimension is stored first (column-major).
pub fn positive_stencils_mut<T>(
    data: &mut [T],
    dim: Index2,
    min: Option<Index2>,
    max: Option<Index2>,
    offset: Option<Index2>, // Stencil offset added to min/max.
) -> impl ParallelIterator<Item = PosStencilMut<T>>
where
    T: Send + Sync,
{
    assert!(
        dim > idx!(0, 0) && dim.iter().fold(1, std::ops::Mul::mul) == data.len(),
        "Wrong dimensions."
    );

    let mut min = min.unwrap_or(Index2::zeros());
    let mut max = max.unwrap_or(dim - idx!(1, 1));

    let offset = offset.unwrap_or(Index2::zeros());
    // Shift all stencils by this offset.
    min += offset;
    max += offset;

    assert!(min >= Index2::zeros() && max < dim);

    let start_y = 0 + min.y * dim.x;

    let it = data[start_y..]
        .par_chunks_exact_mut(2 * dim.x)
        .flat_map(move |row| {
            let (top, bot) = row.split_at_mut(dim.x);
            let y0 = top[min.x..].par_chunks_exact_mut(2);
            let y1 = bot[min.x..].par_chunks_exact_mut(2);

            y0.zip(y1).map(|ys| match ys {
                ([ref mut x0_y0, ref mut x1_y0], [ref mut x0_y1, _]) => PosStencilMut {
                    cell: x0_y0,
                    neighbors: [x1_y0, x0_y1],
                },
                _ => unreachable!(),
            })
        });

    return it;
}

#[test]
fn test() {
    // Grid:
    // 4 5 6
    // 1 2 3
    // -> x
    let mut v = nalgebra::Matrix3x2::<usize>::new(1, 4, 2, 5, 3, 6);
    for s in positive_stencils_mut(v.as_mut_slice(), idx!(3, 2), None, None, None) {
        *s.cell += 3;

        *s.neighbors[0] += 3;
        *s.neighbors[1] += 3;
    }

    assert!(v[(0, 0)] == 4);
    assert!(v[(1, 0)] == 5);
    assert!(v[(0, 1)] == 7);
    assert!(v[(1, 1)] == 5);

    assert!(v[(2, 0)] == 3);
    assert!(v[(2, 1)] == 6);

    print!("{:?}", v.as_slice());
}

#[test]
fn test_parallel() {
    use rayon::prelude::*;

    // Grid:
    // 4 5 6
    // 1 2 3
    // -> x
    let mut v = nalgebra::Matrix3x2::<usize>::new(1, 4, 2, 5, 3, 6);
    let mut stencils: Vec<_> =
        positive_stencils_mut(v.as_mut_slice(), idx!(3, 2), None, None, None).collect();
    stencils.par_iter_mut().for_each(|s| {
        *s.cell += 3;
        *s.neighbors[0] += 3;
        *s.neighbors[1] += 3;
    });

    assert!(v[(0, 0)] == 4);
    assert!(v[(1, 0)] == 5);
    assert!(v[(0, 1)] == 7);
    assert!(v[(1, 1)] == 5);

    assert!(v[(2, 0)] == 3);
    assert!(v[(2, 1)] == 6);

}

#[test]
fn test_without_shift() {
    // Grid:
    // 5 6 7 8
    // 1 2 3 4
    // -> x
    let mut v = nalgebra::Matrix4x2::<usize>::new(1, 5, 2, 6, 3, 7, 4, 8);
    for s in positive_stencils_mut(v.as_mut_slice(), idx!(4, 2), None, None, None) {
        *s.cell += 3;

        *s.neighbors[0] += 3;
        *s.neighbors[1] += 3;
    }

    assert!(v[(0, 0)] == 4);
    assert!(v[(1, 0)] == 5);
    assert!(v[(0, 1)] == 8);
    assert!(v[(1, 1)] == 6);

    assert!(v[(2, 0)] == 6);
    assert!(v[(3, 0)] == 7);
    assert!(v[(2, 1)] == 10);
    assert!(v[(3, 1)] == 8);

    print!("{:?}", v.as_slice());
}

#[test]
fn test_with_shift() {
    // Grid:
    // 0 5 6 7 8
    // 0 1 2 3 4
    // -> x
    let mut v = nalgebra::Matrix5x2::<usize>::new(0, 0, 1, 5, 2, 6, 3, 7, 4, 8);
    for s in positive_stencils_mut(v.as_mut_slice(), idx!(5, 2), Some(idx!(1, 0)), None, None) {
        *s.cell += 3;

        *s.neighbors[0] += 3;
        *s.neighbors[1] += 3;
    }

    assert!(v[(1, 0)] == 4);
    assert!(v[(2, 0)] == 5);
    assert!(v[(1, 1)] == 8);
    assert!(v[(2, 1)] == 6);

    assert!(v[(3, 0)] == 6);
    assert!(v[(4, 0)] == 7);
    assert!(v[(3, 1)] == 10);
    assert!(v[(4, 1)] == 8);

    print!("{:?}", v.as_slice());
}
