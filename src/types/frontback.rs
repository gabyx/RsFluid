#[derive(Clone, Debug)]
pub struct FrontBackBuffer<T> {
    pub front: T, // Back buffer.
    pub back: T,  // Front buffer.
}
