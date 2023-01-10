use super::{Component, Image, Metadata, SingleChannel};

pub struct GrayImage<T: Component> {
    pub data: Vec<T>,
    pub meta: Metadata,
}

impl<T: Component> Image<T> for GrayImage<T> {
    type Channel = SingleChannel;

    fn data(&self) -> &[T] {
        &self.data
    }

    fn data_mut(&mut self) -> &mut [T] {
        &mut self.data
    }

    fn meta(&self) -> &Metadata {
        &self.meta
    }

    fn samples(&self) -> usize {
        1
    }
}
