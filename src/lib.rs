// #![no_std]
#![feature(step_trait)]

// #[macro_use] extern crate derive_more;
// mod memory_structs;
// mod range_inclusive;

mod pervasive;

mod trusted_chunk;

// #[derive(Copy,Clone)]
pub struct Frame {
    pub(crate) number: usize,
}

pub struct RangeInclusive {
    pub(crate) start: usize,
    pub(crate) end: usize
}
impl Clone for RangeInclusive {
    fn clone(&self) -> Self {
        RangeInclusive { start: self.start, end: self.end }
    }
}

pub struct FrameRange(RangeInclusive);

impl Clone for FrameRange {
    fn clone(&self) -> Self {
        FrameRange(self.0.clone())
    }
}

fn main() {}