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
        if true { //self.object_overlaps_in_list(&elem) {
            Some(elem)
        } else {
            self.push(elem);
            None
        }

    }

    fn object_in_list_rec(&self, start_index: usize, obj: &FrameRange) -> (result: Option<usize>) 
        requires 
            self.list.len() > 0,
            start_index < self.list.len(),
            start_index >= 0
        ensures
            result.is_Some() ==> (self.list[result.get_Some_0() as int].0.start == obj.0.start) && (self.list[result.get_Some_0() as int].0.end == obj.0.end),
            // result.is_None() ==> forall|i: int| 
            //     #![trigger obj.0, self.list@.index(i)]
            //     0 <= i <= start_index ==> (self.list@.index(i).0.start != obj.0.start) || (self.list@.index(i).0.end != obj.0.end),
        decreases start_index
    {
        // let list_obj = self.list@.index(start_index);
        if (obj.0.start == self.list.index(start_index).0.start) && (obj.0.end == self.list.index(start_index).0.end) {
            return Some(start_index);
        }

        if start_index == 0 {
            return None;
        }

        return self.object_in_list_rec(start_index - 1, obj);
    }

    fn object_in_list(&self, obj: &FrameRange) -> (result: Option<usize>) 
        ensures
            result.is_Some() ==> (self.list[result.get_Some_0() as int].0.start == obj.0.start) && (self.list[result.get_Some_0() as int].0.end == obj.0.end),
            // result.is_None() ==> forall|i: int| 
            //     0 <= i < self.list.len() ==> (self.list[i].0.start != obj.0.start) || (self.list[i].0.end != obj.0.end),
    {
        let mut i = 0;
        while i < self.list.len() {
            if (self.list.index(i).0.start == obj.0.start) && (self.list.index(i).0.end == obj.0.end) {
                return Some(i);
            }
            i = i+1;
        }
        None
    }
    // proof fn object_overlaps_in_list(&self, start_index: int, obj: &FrameRange) -> (result: Option<int>) 
    //     requires 
    //         self.list.len() > 0,
    //         start_index <= self.list.len(),
    //         start_index >= 0
    //     ensures
    //         result.is_Some() ==> self.list[result.get_Some_0()].0.start == obj.0.start
    //     decreases start_index
    // {
    //     let list_obj = self.list@.index(start_index);
    //     if (obj.0.start == list_obj.0.start) && (obj.0.end == list_obj.0.end) {
    //         return Some(start_index);
    //     }

    //     if start_index == 0 {
    //         return None;
    //     }

    //     return self.object_in_list(start_index - 1, obj);
    // }
}


}