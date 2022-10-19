#![allow(unused_imports)]
use builtin::*;
use builtin_macros::*;
use crate::pervasive::*;
use option::{*, Option::*};
use vec::Vec;

use crate::*;

verus! {

pub struct ChunkList(pub(crate) Vec<FrameRange>);

pub struct TrustedPChunk {
    frames: FrameRange
}

impl TrustedPChunk {
    fn new(frames: FrameRange, chunk_list: &mut ChunkList) -> Option<Self> {
        if frames.0.start > frames.0.end {
            None
        } else {
            Some( TrustedPChunk{frames} )
        }
    }

}

}