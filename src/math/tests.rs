use crate::types::*;

#[cfg(test)]
mod tests {
    use crate::math::*;

    #[test]
    fn check_matrix_mult() {
        let m = Matrix2::new(1.0, 2.0, 3.0, 4.0);
        let v = vec2!(1.0, 2.0);
        let r = m * v;
        let e = vec2!(5.0, 11.0);

        assert!(r == e, "Matrix multiplication not working {} != {}, {}", r, e, m);
    }

    #[test]
    fn check_clamp() {
        let e = Index2::new(3, 8);
        let min = Index2::new(4, 5);
        let max = Index2::new(6, 7);
        let c = clamp_to_range(min, max, e);

        assert!(c == Index2::new(4, 7), "Clamp not working {}", c);
    }
}
