use crate::types::*;
use itertools::Itertools;

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

/// First dimension is stored first (column-major).
pub fn positive_stencils_mut<T>(
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

            // Here the unsafe part happens.
            let cell: *mut T = &mut data[index[0] + index[1] * dim[0]];

            // Get two non-aliasing mutable references for the neighbors.
            return PosStencilMut {
                cell: unsafe { &mut *cell },
                neighbors: PosStencilMut::get_neighbors::<'_>(cell, dim, index),
            };
        });
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
        let mut s = it.next().unwrap();

        // drop(v); // This should invalidate the life-time of `s` but it does not???
        *s.cell += 3;
        if let Some(n) = s.neighbors[0].as_deref_mut() {
            *n += 3;
        };
        if let Some(n) = s.neighbors[1].as_deref_mut() {
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
