#[derive(Clone, Debug)]
pub struct FrontBackBuffer<T> {
    pub front: T, // Back buffer.
    pub back: T,  // Front buffer.
}

impl<T> FrontBackBuffer<T> {
    pub fn swap(&mut self) {
        std::mem::swap(&mut self.front, &mut self.back);
    }
}
