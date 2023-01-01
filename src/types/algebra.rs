use nalgebra;

pub type Scalar = f64;
pub type Vector2 = nalgebra::Vector2<Scalar>;
pub type Matrix2 = nalgebra::Matrix2<Scalar>;
pub type Matrix1x2 = nalgebra::Matrix1x2<Scalar>;
pub type Matrix2x1 = nalgebra::Matrix2x1<Scalar>;

pub type Vector2T<T> = nalgebra::Vector2<T>;
pub type Matrix2T<T> = nalgebra::Matrix2<T>;
pub type Matrix1x2T<T> = nalgebra::Matrix1x2<T>;
pub type Matrix2x1T<T> = nalgebra::Matrix2x1<T>;

pub type Index2 = nalgebra::Vector2<usize>;

#[macro_export]
macro_rules! idx {
    ($x:expr, $($y:expr),+ ) => {
        Index2::new($x, $($y),+)
    };
}

pub use idx as dim;

pub use idx;
