#[derive(Clone, Debug)]
pub struct FrontBackBuffer<T> {
    pub front: T, // Back buffer.
    pub back: T,  // Front buffer.
}

impl<T> FrontBackBuffer<T>  where T: Copy{
    pub fn swap(&mut self) {
        //std::mem::swap(&mut self.front, &mut self.back);

        let temp = self.back;
        self.back = self.front;
        self.front = temp;
    }
}
