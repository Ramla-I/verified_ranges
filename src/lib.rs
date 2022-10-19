// #![no_std]
#![feature(step_trait)]

// #[macro_use] extern crate derive_more;
// mod memory_structs;
// mod range_inclusive;

use std::{ops::Deref, collections::btree_map::Range};

mod pervasive;

mod trusted_chunk;
mod trusted_list;

// #[derive(Copy,Clone)]
pub struct Frame {
    pub(crate) number: usize,
}

pub struct RangeInclusive {
    pub(crate) start: usize,
    pub(crate) end: usize
}
impl RangeInclusive {
    pub fn clone(&self) -> Self {
        RangeInclusive { start: self.start, end: self.end }
    }
}

pub struct FrameRange(RangeInclusive);

impl FrameRange {
    pub fn clone(&self) -> Self {
        FrameRange(self.0.clone())
    }

    pub fn deref(&self) -> &RangeInclusive {
        &self.0
    }

    pub fn overlaps(&self, elem: &Self) -> bool {
        if (self.deref().start > self.deref().end) || (elem.deref().start > elem.deref().end) {
            return false;
        }

        ((elem.deref().end >= self.deref().start) && (elem.deref().start <= self.deref().end)) 
        || 
        ((self.deref().end >= elem.deref().start) && (self.deref().start <= elem.deref().end)) 
    }
}

// impl UniqueObject for FrameRange {
//     fn overlap(&self, elem: &Self) -> bool {
//         if (self.deref().start > self.deref().end) || (elem.deref().start > elem.deref().end) {
//             return false;
//         }

//         ((elem.deref().end >= self.deref().start) && (elem.deref().start <= self.deref().end)) 
//         || 
//         ((self.deref().end >= elem.deref().start) && (self.deref().start <= elem.deref().end)) 
//     }
// }

// pub trait UniqueObject<Rhs = Self>
// where 
//     Rhs: ?Sized,
// {
//     fn overlap(&self, elem: &Rhs) -> bool;
// }

fn main() {}