#![allow(unused_imports)]
use builtin::*;
use builtin_macros::*;
use crate::pervasive::*;
use option::{*, Option::*};
use vec::Vec;

use crate::*;

verus!
{

pub struct TrustedList{
    list: Vec<FrameRange>
}

impl TrustedList {
    pub fn is_empty(&self) -> bool {
        self.list.len() == 0
    }

    pub fn len(&self) -> usize {
        self.list.len()
    }

    fn push(&mut self, elem: FrameRange) {
        self.list.push(elem);
    }

    fn pop(&mut self) -> FrameRange 
        requires 
            old(self).list.len() > 0
    {
        self.list.pop()
    }

    pub fn push_unique(&mut self, elem: FrameRange) -> Option<FrameRange> {
        if self.object_overlaps_in_list(&elem) {
            Some(elem)
        } else {
            self.push(elem);
            None
        }

    }

    fn object_in_list(&self, obj: &FrameRange) -> (result: bool) 
        requires 
            self.list.len() > 0
        ensures
            result ==> exists|i:int| #![trigger self.list[i]] 0 <= i < self.list.len() ==> self.list[i].0.start == obj.0.start
    {
        let mut i = 0;
        while i < self.list.len() {
            let list_obj = self.list.index(i);
            if (obj.0.start == list_obj.0.start) && (obj.0.end == list_obj.0.end) {
                return true;
            }
            i = i + 1;
        }
        false
    }

    fn object_overlaps_in_list(&self, obj: &FrameRange) -> (result: bool)
    {
        let mut i = 0;
        while i < self.list.len() {
            let list_obj = self.list.index(i);
            if obj.overlaps(list_obj) {
                return true;
            }
            i = i + 1;
        }
        false
    }
}
}