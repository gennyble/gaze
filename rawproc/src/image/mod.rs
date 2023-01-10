mod cfa;
mod component;
mod grayimage;
mod hsvimage;
mod rgbimage;
mod sensorimage;

use std::{
    iter::{Skip, StepBy},
    ops::Range,
    slice::{Chunks, ChunksMut, Iter, IterMut},
};

pub use grayimage::GrayImage;
pub use hsvimage::HsvImage;
pub use rgbimage::RgbImage;
pub use sensorimage::SensorImage;

pub use cfa::CFA;
pub use component::{Attribute, Color};

use libraw::Colordata;
use num_traits::{AsPrimitive, Float, Num, PrimInt};

pub struct SingleChannel;
impl Into<usize> for SingleChannel {
    fn into(self) -> usize {
        0
    }
}

pub trait Image<T: Component> {
    type Channel: Into<usize>;

    fn data(&self) -> &[T];
    fn data_mut(&mut self) -> &mut [T];
    fn meta(&self) -> &Metadata;
    fn samples(&self) -> usize;

    fn pixel_range(&self) -> Range<usize> {
        0..(self.meta().width as usize * self.meta().height as usize)
    }

    fn pixel_iter(&self) -> Chunks<'_, T> {
        let samples = self.samples();
        self.data().chunks(samples)
    }

    fn pixel_iter_mut(&mut self) -> ChunksMut<'_, T> {
        let samples = self.samples();
        self.data_mut().chunks_mut(samples)
    }

    fn channel_iter(&self, channel: Self::Channel) -> StepBy<Skip<Iter<'_, T>>> {
        let samples = self.samples();
        self.data().iter().skip(channel.into()).step_by(samples)
    }

    fn channel_iter_mut(&mut self, channel: Self::Channel) -> StepBy<Skip<IterMut<'_, T>>> {
        let samples = self.samples();
        self.data_mut()
            .iter_mut()
            .skip(channel.into())
            .step_by(samples)
    }
}

#[derive(Clone, Debug)]
pub struct Metadata {
    pub width: u32,
    pub height: u32,
    pub cfa: CFA,
    pub bit_depth: u8,
    pub colordata: Colordata,
}

impl Metadata {
    pub fn new(width: u32, height: u32, cfa: CFA, colordata: Colordata) -> Self {
        Self {
            width,
            height,
            cfa,
            bit_depth: 14, //TODO: Allow changing bit depth
            colordata,
        }
    }

    pub fn xytoi(&self, x: u32, y: u32) -> usize {
        (y * self.width + x) as usize
    }

    pub fn itoxy(&self, i: usize) -> (u32, u32) {
        let y = i / self.width as usize;
        let x = i % self.width as usize;

        (x as u32, y as u32)
    }

    pub fn pixels(&self) -> usize {
        self.width as usize * self.height as usize
    }

    pub fn color_at_index(&self, index: usize) -> Color {
        let (x, y) = self.itoxy(index);
        self.cfa.color_at(x, y)
    }

    pub fn color_at_xy(&self, x: u32, y: u32) -> Color {
        self.cfa.color_at(x, y)
    }

    pub fn depth_max(&self) -> usize {
        2.pow(self.bit_depth as u32) - 1
    }
}

pub trait Component: PartialEq + PartialOrd + Num + Copy {}
impl<T: PartialEq + PartialOrd + Num + Copy> Component for T {}

pub trait IntComponent: Component + PrimInt + AsPrimitive<f32> {}
impl<T: Component + PrimInt + AsPrimitive<f32>> IntComponent for T {}

pub trait FloatComponent: Component + Float + AsPrimitive<u8> + AsPrimitive<u16> {}
impl<T: Component + Float + AsPrimitive<u8> + AsPrimitive<u16>> FloatComponent for T {}

#[inline]
pub(crate) fn data_to_floats<I: IntComponent>(meta: &Metadata, data: Vec<I>) -> Vec<f32> {
    let max = meta.depth_max() as f32;

    data.into_iter().map(|u| u.as_() / max).collect()
}

#[inline]
pub(crate) fn data_to_u8s(meta: &Metadata, data: Vec<f32>) -> Vec<u8> {
    let max = meta.depth_max().min(u8::MAX as usize) as f32;

    data.into_iter().map(|u| (u * max) as u8).collect()
}

#[inline]
pub(crate) fn data_to_u16s(meta: &Metadata, data: Vec<f32>) -> Vec<u16> {
    let max = meta.depth_max().min(u16::MAX as usize) as f32;

    data.into_iter().map(|u| (u * max) as u16).collect()
}
