use crate::types::*;
use nalgebra;

pub fn clamp_to_range<T>(min: Vector2T<T>, max: Vector2T<T>, index: Vector2T<T>) -> Vector2T<T>
where
    T: nalgebra::Scalar + PartialOrd + Copy,
{
    return Vector2T::<T>::new(
        nalgebra::clamp(index.x, min.x, max.x),
        nalgebra::clamp(index.y, min.y, max.y),
    );
}
