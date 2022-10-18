// use bit_field::BitField;
use core::{
    cmp::{min, max},
    fmt,
    iter::Step,
    ops::{Add, AddAssign, Deref, DerefMut, Sub, SubAssign},
};
// use zerocopy::FromBytes;

use crate::{addr::*, unit::*, range_inclusive::*};

/// A range of [`Frame`]s that are contiguous in physical memory.
#[derive(Clone, PartialEq, Eq)]
pub struct FrameRange(RangeInclusive<Frame>);

impl FrameRange {
    /// Creates a new range of [`Frame`]s that spans from `start` to `end`, both inclusive bounds.
    pub const fn new(start: Frame, end: Frame) -> FrameRange {
        FrameRange(RangeInclusive::new(start, end))
    }

    /// Creates a `FrameRange` that will always yield `None` when iterated.
    pub const fn empty() -> FrameRange {
        FrameRange::new(Frame { number: 1 }, Frame { number: 0 })
    }

    /// A convenience method for creating a new `FrameRange` that spans
    /// all [`Frame`]s from the given [`PhysicalAddress`] to an end bound based on the given size.
    pub fn from_phys_addr(starting_addr: PhysicalAddress, size_in_bytes: usize) -> FrameRange {
        assert!(size_in_bytes > 0);
        let start = Frame::containing_address(starting_addr);
        // The end bound is inclusive, hence the -1. Parentheses are needed to avoid overflow.
        let end = Frame::containing_address(starting_addr + (size_in_bytes - 1));
        FrameRange::new(start, end)
    }

    /// Returns the [`PhysicalAddress`] of the starting [`Frame`] in this `FrameRange`.
    pub const fn start_address(&self) -> PhysicalAddress {
        self.0.start().start_address()
    }

    /// Returns the number of [`Frame`]s covered by this iterator.
    /// Use this instead of [`Iterator::count()`] method.
    /// This is instant, because it doesn't need to iterate over each entry, unlike normal iterators.
    pub const fn size_in_frames(&self) -> usize {
        // add 1 because it's an inclusive range
        (self.0.end().number + 1).saturating_sub(self.0.start().number)
    }

    /// Returns the size of this range in number of bytes.
    pub const fn size_in_bytes(&self) -> usize {
        self.size_in_frames() * PAGE_SIZE
    }

    /// Returns `true` if this `FrameRange` contains the given [`PhysicalAddress`].
    pub fn contains_address(&self, addr: PhysicalAddress) -> bool {
        self.0.contains(&Frame::containing_address(addr))
    }

    /// Returns the offset of the given [`PhysicalAddress`] within this `FrameRange`,
    /// i.e., `addr - self.start_address()`.
    /// If the given `addr` is not covered by this range of [`Frame`]s, this returns `None`.
    /// # Examples
    /// If the range covers addresses `0x2000` to `0x4000`, then `offset_of_address(0x3500)` would return `Some(0x1500)`.
    pub fn offset_of_address(&self, addr: PhysicalAddress) -> Option<usize> {
        if self.contains_address(addr) {
            Some(addr.value() - self.start_address().value())
        } else {
            None
        }
    }

    /// Returns the [`PhysicalAddress`] at the given `offset` into this `FrameRange` within this `FrameRange`,
    /// i.e., `addr - self.start_address()`.\n\n \
    /// If the given `offset` is not within this range of [`Frame`]s, this returns `None`.\n\n \
    /// # Examples\n \
    /// If the range covers addresses `0x2000` to `0x4000`, then `address_at_offset(0x1500)` would return `Some(0x3500)`.
    pub fn address_at_offset(&self, offset: usize) -> Option<PhysicalAddress> {
        if offset <= self.size_in_bytes() {
            Some(self.start_address() + offset)
        }
        else {
            None
        }
    }

    /// "Returns a new separate `FrameRange` that is extended to include the given [`Frame`].
    pub fn to_extended(&self, to_include: Frame) -> FrameRange {
        // if the current range was empty, return a new range containing only the given page/frame
        if self.is_empty() {
            return FrameRange::new(to_include.clone(), to_include);
        }
        let start = core::cmp::min(self.0.start(), &to_include);
        let end = core::cmp::max(self.0.end(), &to_include);
        FrameRange::new(start.clone(), end.clone())
    }

    /// "Returns an inclusive `FrameRange` representing the [`Frame`]s that overlap \
    /// across this `FrameRange` and the given other `FrameRange`.\n\n \
    /// If there is no overlap between the two ranges, `None` is returned.
    pub fn overlap(&self, other: &FrameRange) -> Option<FrameRange> {
        let starts = max(*self.start(), *other.start());
        let ends   = min(*self.end(),   *other.end());
        if starts <= ends {
            Some(FrameRange::new(starts, ends))
        } else {
            None
        }
    }
}
impl fmt::Debug for FrameRange {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self.0)
    }
}
impl Deref for FrameRange {
    type Target = RangeInclusive<Frame>;
    fn deref(&self) -> &RangeInclusive<Frame> {
        &self.0
    }
}
impl DerefMut for FrameRange {
    fn deref_mut(&mut self) -> &mut RangeInclusive<Frame> {
        &mut self.0
    }
}
impl<'a> IntoIterator for &'a FrameRange {
    type Item = Frame;
    type IntoIter = RangeInclusiveIterator<Frame>;
    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}



/// A range of [`Page`]s that are contiguous in virtual memory.
#[derive(Clone, PartialEq, Eq)]
pub struct PageRange(RangeInclusive<Page>);

impl PageRange {
    /// Creates a new range of [`Page`]s that spans from `start` to `end`, both inclusive bounds.
    pub const fn new(start: Page, end: Page) -> PageRange {
        PageRange(RangeInclusive::new(start, end))
    }

    /// Creates a `PageRange` that will always yield `None` when iterated.
    pub const fn empty() -> PageRange {
        PageRange::new(Page { number: 1 }, Page { number: 0 })
    }

    /// A convenience method for creating a new `PageRange` that spans
    /// all [`Page`]s from the given [`VirtualAddress`] to an end bound based on the given size.
    pub fn from_virt_addr(starting_addr: VirtualAddress, size_in_bytes: usize) -> PageRange {
        assert!(size_in_bytes > 0);
        let start = Page::containing_address(starting_addr);
        // The end bound is inclusive, hence the -1. Parentheses are needed to avoid overflow.
        let end = Page::containing_address(starting_addr + (size_in_bytes - 1));
        PageRange::new(start, end)
    }

    /// Returns the [`VirtualAddress`] of the starting [`Page`] in this `PageRange`.
    pub const fn start_address(&self) -> VirtualAddress {
        self.0.start().start_address()
    }

    /// Returns the number of [`Page`]s covered by this iterator.\n\n \
    /// Use this instead of [`Iterator::count()`] method. \
    /// This is instant, because it doesn't need to iterate over each entry, unlike normal iterators.
    pub const fn size_in_pages(&self) -> usize {
        // add 1 because it's an inclusive range
        (self.0.end().number + 1).saturating_sub(self.0.start().number)
    }

    /// Returns the size of this range in number of bytes.
    pub const fn size_in_bytes(&self) -> usize {
        self.size_in_pages() * PAGE_SIZE
    }

    /// Returns `true` if this `PageRange` contains the given [`VirtualAddress`].
    pub fn contains_address(&self, addr: VirtualAddress) -> bool {
        self.0.contains(&Page::containing_address(addr))
    }

    /// Returns the offset of the given [`VirtualAddress`] within this `PageRange`, \
    /// i.e., `addr - self.start_address()`.\n\n \
    /// If the given `addr` is not covered by this range of [`Page`]s, this returns `None`.\n\n \
    /// # Examples\n \
    /// If the range covers addresses `0x2000` to `0x4000`, then `offset_of_address(0x3500)` would return `Some(0x1500)`.
    pub fn offset_of_address(&self, addr: VirtualAddress) -> Option<usize> {
        if self.contains_address(addr) {
            Some(addr.value() - self.start_address().value())
        } else {
            None
        }
    }

    /// Returns the [`VirtualAddress`] at the given `offset` into this `PageRange`within this `PageRange`, \
    /// i.e., `addr - self.start_address()`.\n\n \
    /// If the given `offset` is not within this range of [`Page`]s, this returns `None`.\n\n \
    /// # Examples\n \
    /// If the range covers addresses `0x2000` to `0x4000`, then `address_at_offset(0x1500)` would return `Some(0x3500)`.
    pub fn address_at_offset(&self, offset: usize) -> Option<VirtualAddress> {
        if offset <= self.size_in_bytes() {
            Some(self.start_address() + offset)
        }
        else {
            None
        }
    }

    /// Returns a new separate `PageRange` that is extended to include the given [`Page`].
    pub fn to_extended(&self, to_include: Page) -> PageRange {
        // if the current range was empty, return a new range containing only the given page/frame
        if self.is_empty() {
            return PageRange::new(to_include.clone(), to_include);
        }
        let start = core::cmp::min(self.0.start(), &to_include);
        let end = core::cmp::max(self.0.end(), &to_include);
        PageRange::new(start.clone(), end.clone())
    }

    /// Returns an inclusive `PageRange` representing the [`Page`]s that overlap \
    /// across this `PageRange` and the given other `PageRange`.\n\n \
    /// If there is no overlap between the two ranges, `None` is returned.
    pub fn overlap(&self, other: &PageRange) -> Option<PageRange> {
        let starts = max(*self.start(), *other.start());
        let ends   = min(*self.end(),   *other.end());
        if starts <= ends {
            Some(PageRange::new(starts, ends))
        } else {
            None
        }
    }
}
impl fmt::Debug for PageRange {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self.0)
    }
}
impl Deref for PageRange {
    type Target = RangeInclusive<Page>;
    fn deref(&self) -> &RangeInclusive<Page> {
        &self.0
    }
}
impl DerefMut for PageRange {
    fn deref_mut(&mut self) -> &mut RangeInclusive<Page> {
        &mut self.0
    }
}
impl <'a>IntoIterator for &'a PageRange {
    type Item = Page;
    type IntoIter = RangeInclusiveIterator<Page>;
    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

