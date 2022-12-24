struct FrontBackBuffer<T> {
    pub front: T, // Back buffer.
    pub back: T,  // Front buffer.
}

type Velocity = (f64, f64);
type Field<T, const WIDTH: usize, const HEIGHT: usize> = [[T; WIDTH]; HEIGHT];

pub struct Grid<const DIM_X: usize, const DIM_Y: usize> {
    pub cell_width: f64,

    pub density: Field<f64, DIM_X, DIM_Y>,
    // pub velocity: FrontBackBuffer<Field<Velocity, DIM_X, DIM_Y>>,
    // pub pressure: Field<f64, DIM_X, DIM_Y>,
    // pub smoke: FrontBackBuffer<Field<f64, DIM_X, DIM_Y>>,
}

impl<const DIM_X: usize, const DIM_Y: usize> Grid<DIM_X, DIM_Y> {

    pub fn size(&self) -> (usize, usize) {
        return (DIM_X, DIM_Y);
    }

    // fn velocity(&self) -> (f64, f64) {
    //     return self.velocity.front[1][2];
    // }

    pub fn new() -> Self {
       return Grid{ cell_width: 10.0, density: [[0.0; DIM_X]; DIM_Y] };
    }

}
