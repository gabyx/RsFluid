use crate::math::*;
use crate::scene::grid_stencil::PosStencilMut;
use crate::types::*;
use itertools::Itertools;
use rayon::prelude::*;

/// First dimension is stored first (column-major).
pub fn positive_stencils_mut<T>(
    data: &mut [T],
    dim: Index2,
    min: Option<Index2>,
    max: Option<Index2>,
    offset: Option<Index2>, // Stencil offset added to min/max.
) -> impl ParallelIterator<Item = PosStencilMut<'_, T>>
where
    T: Send + Sync,
{
    assert!(
        dim > idx!(0, 0) && dim.iter().fold(1, std::ops::Mul::mul) == data.len(),
        "Wrong dimensions."
    );

    let mut min = min.unwrap_or(Index2::zeros());
    let mut max = max.unwrap_or(dim);

    let offset = offset.unwrap_or(Index2::zeros());
    // Shift all stencils by this offset.
    min += offset;
    max = clamp_to_range(idx!(0, 0), dim, max + offset);

    for dir in 0..2 {
        max[dir] -= (max[dir] - min[dir]) % 2 // Subtract the remainder to make the range correct.
    }

    assert!(
        min >= Index2::zeros() && max <= dim && min < max,
        "Min: {} and max: {}, dim: {}",
        min,
        max,
        dim
    );

    return (min[0]..max[0])
        .step_by(2)
        .cartesian_product((min[1]..max[1]).step_by(2))
        .map(move |(i, j)| {
            let index = idx!(i, j);

            // Here the unsafe part happens.
            let offset = index[0] + index[1] * dim[0];
            let cell: *mut T = unsafe { data.as_mut_ptr().add(offset) };

            // Get two non-aliasing mutable references for the neighbors.
            return PosStencilMut {
                cell: unsafe { &mut *cell },
                neighbors: [unsafe { &mut *cell.add(1) }, unsafe {
                    &mut *cell.add(dim[0])
                }],
            };
        }).par_bridge();

}

#[test]
fn test() {
    // Grid:
    // 3 4
    // 1 2
    // -> x
    let mut v = Matrix2T::<usize>::new(1, 3, 2, 4);
    {
        let mut it = positive_stencils_mut(v.as_mut_slice(), idx!(2, 2), None, None);
        let s = it.next().unwrap();

        // drop(v); // This should invalidate the life-time of `s`
        *s.cell += 3;
        if let Some(&mut ref mut n) = s.neighbors[0] {
            *n += 3;
        };
        if let Some(&mut ref mut n) = s.neighbors[1] {
            *n += 3;
        };
    }

    assert!(v[(0, 0)] == 4);
    assert!(v[(1, 0)] == 5);
    assert!(v[(0, 1)] == 6);

    {
        let mut it = positive_stencils_mut(v.as_mut_slice(), idx!(2, 2), None, None);
        it.next();
        assert!(it.next().is_none());
    }
}
