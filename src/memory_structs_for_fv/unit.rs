// use bit_field::BitField;
use core::{
    cmp::{min, max},
    fmt,
    iter::Step,
    ops::{Add, AddAssign, Deref, DerefMut, RangeInclusive, Sub, SubAssign},
};
// use zerocopy::FromBytes;

use crate::memory_structs::addr::*;

/// A `" Frame "` is a chunk of **" physical "** memory aligned to a [`PAGE_SIZE`] boundary.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Frame {
    pub(crate) number: usize,
}

impl Frame {
    /// Returns the `" PhysicalAddress "` at the start of this `" Frame "`.
    pub const fn start_address(&self) -> PhysicalAddress {
        PhysicalAddress::new_canonical(self.number * PAGE_SIZE)
    }

    /// Returns the number of this `" Frame "`.
    #[inline(always)]
    pub const fn number(&self) -> usize {
        self.number
    }
    
    /// Returns the `" Frame "` containing the given `" PhysicalAddress "`.
    pub const fn containing_address(addr: PhysicalAddress) -> Frame {
        Frame {
            number: addr.value() / PAGE_SIZE,
        }
    }
}
// impl fmt::Debug for Frame {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         write!(f, concat!(stringify!(Frame), "(", "p", "{:#X})"), self.start_address())
//     }
// }
impl Add<usize> for Frame {
    type Output = Frame;
    fn add(self, rhs: usize) -> Frame {
        // cannot exceed max page number (which is also max frame number)
        Frame {
            number: core::cmp::min(MAX_PAGE_NUMBER, self.number.saturating_add(rhs)),
        }
    }
}
impl AddAssign<usize> for Frame {
    fn add_assign(&mut self, rhs: usize) {
        *self = Frame {
            number: core::cmp::min(MAX_PAGE_NUMBER, self.number.saturating_add(rhs)),
        };
    }
}
impl Sub<usize> for Frame {
    type Output = Frame;
    fn sub(self, rhs: usize) -> Frame {
        Frame {
            number: self.number.saturating_sub(rhs),
        }
    }
}
impl SubAssign<usize> for Frame {
    fn sub_assign(&mut self, rhs: usize) {
        *self = Frame {
            number: self.number.saturating_sub(rhs),
        };
    }
}
/// Implementing `Step` allows `" Frame "` to be used in an [`Iterator`].
impl Step for Frame {
    #[inline]
    fn steps_between(start: &Frame, end: &Frame) -> Option<usize> {
        Step::steps_between(&start.number, &end.number)
    }
    #[inline]
    fn forward_checked(start: Frame, count: usize) -> Option<Frame> {
        Step::forward_checked(start.number, count).map(|n| Frame { number: n })
    }
    #[inline]
    fn backward_checked(start: Frame, count: usize) -> Option<Frame> {
        Step::backward_checked(start.number, count).map(|n| Frame { number: n })
    }
}

/// A `Page` is a chunk of **" virtual "** memory aligned to a [`PAGE_SIZE`] boundary.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Page {
    pub(crate) number: usize,
}

impl Page {
    /// Returns the `" VirtualAddress "` at the start of this `Page`.
    pub const fn start_address(&self) -> VirtualAddress {
        VirtualAddress::new_canonical(self.number * PAGE_SIZE)
    }

    /// Returns the number of this `Page`.
    #[inline(always)]
    pub const fn number(&self) -> usize {
        self.number
    }
    
    /// Returns the `Page` containing the given `" VirtualAddress "`.
    pub const fn containing_address(addr: VirtualAddress) -> Page {
        Page {
            number: addr.value() / PAGE_SIZE,
        }
    }
}
// impl fmt::Debug for Page {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         write!(f, concat!(stringify!(Page), "(", "v", "{:#X})"), self.start_address())
//     }
// }
impl Add<usize> for Page {
    type Output = Page;
    fn add(self, rhs: usize) -> Page {
        // cannot exceed max page number (which is also max frame number)
        Page {
            number: core::cmp::min(MAX_PAGE_NUMBER, self.number.saturating_add(rhs)),
        }
    }
}
impl AddAssign<usize> for Page {
    fn add_assign(&mut self, rhs: usize) {
        *self = Page {
            number: core::cmp::min(MAX_PAGE_NUMBER, self.number.saturating_add(rhs)),
        };
    }
}
impl Sub<usize> for Page {
    type Output = Page;
    fn sub(self, rhs: usize) -> Page {
        Page {
            number: self.number.saturating_sub(rhs),
        }
    }
}
impl SubAssign<usize> for Page {
    fn sub_assign(&mut self, rhs: usize) {
        *self = Page {
            number: self.number.saturating_sub(rhs),
        };
    }
}
/// Implementing `Step` allows `Page` to be used in an [`Iterator`].
impl Step for Page {
    #[inline]
    fn steps_between(start: &Page, end: &Page) -> Option<usize> {
        Step::steps_between(&start.number, &end.number)
    }
    #[inline]
    fn forward_checked(start: Page, count: usize) -> Option<Page> {
        Step::forward_checked(start.number, count).map(|n| Page { number: n })
    }
    #[inline]
    fn backward_checked(start: Page, count: usize) -> Option<Page> {
        Step::backward_checked(start.number, count).map(|n| Page { number: n })
    }
}


// Implement other functions for the `Page` type that aren't relevant for `Frame.
impl Page {
    /// Returns the 9-bit part of this `Page`'s [`VirtualAddress`] that is the index into the P4 page table entries list.
    pub const fn p4_index(&self) -> usize {
        (self.number >> 27) & 0x1FF
    }

    /// Returns the 9-bit part of this `Page`'s [`VirtualAddress`] that is the index into the P3 page table entries list.
    pub const fn p3_index(&self) -> usize {
        (self.number >> 18) & 0x1FF
    }

    /// Returns the 9-bit part of this `Page`'s [`VirtualAddress`] that is the index into the P2 page table entries list.
    pub const fn p2_index(&self) -> usize {
        (self.number >> 9) & 0x1FF
    }

    /// Returns the 9-bit part of this `Page`'s [`VirtualAddress`] that is the index into the P1 page table entries list.
    ///
    /// Using this returned `usize` value as an index into the P1 entries list will give you the final PTE,
    /// from which you can extract the mapped [`Frame`]  using `PageTableEntry::pointed_frame()`.
    pub const fn p1_index(&self) -> usize {
        (self.number >> 0) & 0x1FF
    }
}
