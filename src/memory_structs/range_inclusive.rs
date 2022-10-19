use core::iter::Step;
use core::ops::{RangeBounds, Bound, Bound::Included};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RangeInclusive<Idx: Clone + PartialOrd> {
    pub(crate) start: Idx,
    pub(crate) end: Idx
}

impl<Idx: Clone + PartialOrd> RangeInclusive<Idx> {
    #[inline]
    pub const fn new(start: Idx, end: Idx) -> Self {
        Self{ start, end }
    }

    #[inline]
    pub const fn start(&self) -> &Idx {
        &self.start
    }

    #[inline]
    pub const fn end(&self) -> &Idx {
        &self.end
    }

    #[inline]
    pub fn into_inner(self) -> (Idx, Idx) {
        (self.start, self.end)
    }

    pub fn iter(&self) -> RangeInclusiveIterator<Idx> {
        RangeInclusiveIterator { offset: self.start.clone(), end: self.end.clone() }
    }

    pub fn is_empty(&self) -> bool {
        self.end < self.start
    }

    pub fn contains<U>(&self, item: &U) -> bool
    where
        Idx: PartialOrd<U>,
        U: ?Sized + PartialOrd<Idx>,
    {
        <Self as RangeBounds<Idx>>::contains(self, item)
    }
}

impl<T: Clone + PartialOrd> RangeBounds<T> for RangeInclusive<T> {
    fn start_bound(&self) -> Bound<&T> {
        Included(&self.start)
    }
    fn end_bound(&self) -> Bound<&T> {
        Included(&self.end)
    }
}

impl<'a, Idx: Clone + PartialOrd + Step> IntoIterator for &'a RangeInclusive<Idx> {
    type Item = Idx;
    type IntoIter = RangeInclusiveIterator<Idx>;

    fn into_iter(self) -> RangeInclusiveIterator<Idx> {
        self.iter()
    }
}

pub struct RangeInclusiveIterator<Idx> {
    offset: Idx,
    end: Idx
}


impl<A: Step> Iterator for RangeInclusiveIterator<A> {
    type Item = A;

    fn next(&mut self) -> Option<Self::Item> {
        if self.offset > self.end {
            None
        } else {
            let n = Step::forward_checked(self.offset.clone(), 1).expect("`Step` invariants not upheld");
            Some(core::mem::replace(&mut self.offset, n))
        }
    }
}