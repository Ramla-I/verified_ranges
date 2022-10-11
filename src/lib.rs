//! This crate contains common types used for memory mapping. 

#![no_std]
#![feature(step_trait)]

extern crate bit_field;
#[macro_use] extern crate bitflags;


use bit_field::BitField;
use core::{
    cmp::{min, max},
    fmt,
    iter::Step,
    ops::{Add, AddAssign, Deref, DerefMut, RangeInclusive, Sub, SubAssign},
};


pub const MAX_VIRTUAL_ADDRESS: usize = usize::MAX;

/// The lower 12 bits of a virtual address correspond to the P1 page frame offset. 
pub const PAGE_SHIFT: usize = 12;
/// Page size is 4096 bytes, 4KiB pages.
pub const PAGE_SIZE: usize = 1 << PAGE_SHIFT;

pub const MAX_PAGE_NUMBER: usize = MAX_VIRTUAL_ADDRESS / PAGE_SIZE;

/// A mask for the bits of a page table entry that contain the physical frame address.
pub const PAGE_TABLE_ENTRY_FRAME_MASK: u64 = 0x000_FFFFFFFFFF_000;

bitflags! {
    /// Page table entry flags on the x86_64 architecture. 
    /// 
    /// The designation of bits in each `PageTableEntry` is as such:
    /// * Bits `[0:8]` (inclusive) are reserved by hardware for access flags.
    /// * Bits `[9:11]` (inclusive) are available for custom OS usage.
    /// * Bits `[12:51]` (inclusive) are reserved by hardware to hold the physical frame address.
    /// * Bits `[52:62]` (inclusive) are available for custom OS usage.
    /// * Bit  `63` is reserved by hardware for access flags (noexec).
    /// 
    pub struct EntryFlags: u64 {
        /// If set, this page is currently "present" in memory. 
        /// If not set, this page is not in memory, e.g., not mapped, paged to disk, etc.
        const PRESENT           = 1 <<  0;
        /// If set, writes to this page are allowed.
        /// If not set, this page is read-only.
        const WRITABLE          = 1 <<  1;
        /// If set, userspace (ring 3) can access this page.
        /// If not set, only kernelspace (ring 0) can access this page. 
        const USER_ACCESSIBLE   = 1 <<  2;
        /// If set, writes to this page go directly through the cache to memory. 
        const WRITE_THROUGH     = 1 <<  3;
        /// If set, this page's content is never cached, neither for read nor writes. 
        const NO_CACHE          = 1 <<  4;
        /// The hardware will set this bit when the page is accessed.
        const ACCESSED          = 1 <<  5;
        /// The hardware will set this bit when the page has been written to.
        const DIRTY             = 1 <<  6;
        /// Set this bit if this page table entry represents a "huge" page. 
        /// This bit may be used as follows:
        /// * For a P4-level PTE, it must be not set. 
        /// * If set for a P3-level PTE, it means this PTE maps a 1GiB huge page.
        /// * If set for a P2-level PTE, it means this PTE maps a 1MiB huge page.
        /// * For a P1-level PTE, it must be not set. 
        const HUGE_PAGE         = 1 <<  7;
        /// Set this bit to indicate that this page is mapped across all address spaces 
        /// (all root page tables) and doesn't need to be flushed out of the TLB 
        /// when switching to another page table. 
        const GLOBAL            = 0 <<  8; // 1 <<  8; // Currently disabling GLOBAL bit.

        /// Set this bit to indicate that the frame pointed to by this page table entry
        /// is owned **exclusively** by that page table entry.
        /// Currently, in Theseus, we only set the `EXCLUSIVE` bit for P1-level PTEs
        /// that we **know** are bijective (1-to-1 virtual-to-physical) mappings. 
        /// If this bit is set, the pointed frame will be safely deallocated
        /// once this page table entry is unmapped. 
        const EXCLUSIVE         = 1 <<  9;

        /// Set this bit to forbid execution of the mapped page.
        /// In other words, if you want the page to be executable, do NOT set this bit. 
        const NO_EXECUTE        = 1 << 63;
    }
}



/// A physical memory address, which is a `usize` under the hood
#[derive(
    Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default, 
    Binary, Octal, LowerHex, UpperHex, 
    BitAnd, BitOr, BitXor, BitAndAssign, BitOrAssign, BitXorAssign, 
    Add, Sub, AddAssign, SubAssign,
    FromBytes,
)]
#[repr(transparent)]
pub struct PhysicalAddress(usize);

impl PhysicalAddress {
    /// Creates a new `PhysicalAddress`, returning an error if the address is not canonical.
    /// This is useful for checking whether an address is valid before using it. 
    /// For example, on x86_64, virtual addresses are canonical
    /// if their upper bits `(64:48]` are sign-extended from bit 47,
    /// and physical addresses are canonical if their upper bits `(64:52]` are 0.
    pub fn new(addr: usize) -> Option<PhysicalAddress> {
        if is_canonical_physical_address(addr) { Some(PhysicalAddress(addr)) } else { None }
    }

    /// Creates a new `PhysicalAddress` that is guaranteed to be canonical.
    pub const fn new_canonical(addr: usize) -> PhysicalAddress {
        PhysicalAddress(canonicalize_physical_address(addr))
    }

    /// Creates a new `PhysicalAddress` with a value 0.
    pub const fn zero() -> PhysicalAddress {
        PhysicalAddress(0)
    }

    /// Returns the underlying `usize` value for this `PhysicalAddress`.
    #[inline]
    pub const fn value(&self) -> usize {
        self.0
    }

    /// Returns the offset from the " frame " boundary specified by this `PhysicalAddress`.
    // For example, if the [`PAGE_SIZE`] is 4096 (4KiB), then this will return
    // the least significant 12 bits `(12:0]` of this `PhysicalAddress`.
    pub const fn [<frame _offset>](&self) -> usize {
        self.0 & (PAGE_SIZE - 1)
    }
}
impl fmt::Debug for PhysicalAddress {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, concat!(p, "{:#X}"), self.0)
    }
}
impl fmt::Display for PhysicalAddress {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}
impl fmt::Pointer for PhysicalAddress {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}
impl Add<usize> for PhysicalAddress {
    type Output = PhysicalAddress;
    fn add(self, rhs: usize) -> PhysicalAddress {
        PhysicalAddress::new_canonical(self.0.saturating_add(rhs))
    }
}
impl AddAssign<usize> for PhysicalAddress {
    fn add_assign(&mut self, rhs: usize) {
        *self = PhysicalAddress::new_canonical(self.0.saturating_add(rhs));
    }
}
impl Sub<usize> for PhysicalAddress {
    type Output = PhysicalAddress;
    fn sub(self, rhs: usize) -> PhysicalAddress {
        PhysicalAddress::new_canonical(self.0.saturating_sub(rhs))
    }
}
impl SubAssign<usize> for PhysicalAddress {
    fn sub_assign(&mut self, rhs: usize) {
        *self = PhysicalAddress::new_canonical(self.0.saturating_sub(rhs));
    }
}
impl Into<usize> for PhysicalAddress {
    #[inline]
    fn into(self) -> usize {
        self.0
    }
}


/// A macro for defining `VirtualAddress` and `PhysicalAddress` structs
/// and implementing their common traits, which are generally identical.
macro_rules! implement_address {
    ($TypeName:ident, $desc:literal, $prefix:literal, $is_canonical:ident, $canonicalize:ident, $chunk:ident) => {
        paste! { // using the paste crate's macro for easy concatenation

            #[doc = "A " $desc " memory address, which is a `usize` under the hood."]
            #[derive(
                Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default, 
                Binary, Octal, LowerHex, UpperHex, 
                BitAnd, BitOr, BitXor, BitAndAssign, BitOrAssign, BitXorAssign, 
                Add, Sub, AddAssign, SubAssign,
                FromBytes,
            )]
            #[repr(transparent)]
            pub struct $TypeName(usize);

            impl $TypeName {
                #[doc = "Creates a new `" $TypeName "`, returning an error if the address is not canonical.\n\n \
                    This is useful for checking whether an address is valid before using it. 
                    For example, on x86_64, virtual addresses are canonical
                    if their upper bits `(64:48]` are sign-extended from bit 47,
                    and physical addresses are canonical if their upper bits `(64:52]` are 0."]
                pub fn new(addr: usize) -> Option<$TypeName> {
                    if $is_canonical(addr) { Some($TypeName(addr)) } else { None }
                }

                #[doc = "Creates a new `" $TypeName "` that is guaranteed to be canonical."]
                pub const fn new_canonical(addr: usize) -> $TypeName {
                    $TypeName($canonicalize(addr))
                }

                #[doc = "Creates a new `" $TypeName "` with a value 0."]
                pub const fn zero() -> $TypeName {
                    $TypeName(0)
                }

                #[doc = "Returns the underlying `usize` value for this `" $TypeName "`."]
                #[inline]
                pub const fn value(&self) -> usize {
                    self.0
                }

                #[doc = "Returns the offset from the " $chunk " boundary specified by this `"
                    $TypeName ".\n\n \
                    For example, if the [`PAGE_SIZE`] is 4096 (4KiB), then this will return
                    the least significant 12 bits `(12:0]` of this `" $TypeName "`."]
                pub const fn [<$chunk _offset>](&self) -> usize {
                    self.0 & (PAGE_SIZE - 1)
                }
            }
            impl fmt::Debug for $TypeName {
                fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                    write!(f, concat!($prefix, "{:#X}"), self.0)
                }
            }
            impl fmt::Display for $TypeName {
                fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                    write!(f, "{:?}", self)
                }
            }
            impl fmt::Pointer for $TypeName {
                fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                    write!(f, "{:?}", self)
                }
            }
            impl Add<usize> for $TypeName {
                type Output = $TypeName;
                fn add(self, rhs: usize) -> $TypeName {
                    $TypeName::new_canonical(self.0.saturating_add(rhs))
                }
            }
            impl AddAssign<usize> for $TypeName {
                fn add_assign(&mut self, rhs: usize) {
                    *self = $TypeName::new_canonical(self.0.saturating_add(rhs));
                }
            }
            impl Sub<usize> for $TypeName {
                type Output = $TypeName;
                fn sub(self, rhs: usize) -> $TypeName {
                    $TypeName::new_canonical(self.0.saturating_sub(rhs))
                }
            }
            impl SubAssign<usize> for $TypeName {
                fn sub_assign(&mut self, rhs: usize) {
                    *self = $TypeName::new_canonical(self.0.saturating_sub(rhs));
                }
            }
            impl Into<usize> for $TypeName {
                #[inline]
                fn into(self) -> usize {
                    self.0
                }
            }
        }
    };
}

#[inline]
fn is_canonical_virtual_address(virt_addr: usize) -> bool {
    match virt_addr.get_bits(47..64) {
        0 | 0b1_1111_1111_1111_1111 => true,
        _ => false,
    }
}

#[inline]
const fn canonicalize_virtual_address(virt_addr: usize) -> usize {
    // match virt_addr.get_bit(47) {
    //     false => virt_addr.set_bits(48..64, 0),
    //     true =>  virt_addr.set_bits(48..64, 0xffff),
    // };

    // The below code is semantically equivalent to the above, but it works in const functions.
    ((virt_addr << 16) as isize >> 16) as usize
}

#[inline]
fn is_canonical_physical_address(phys_addr: usize) -> bool {
    match phys_addr.get_bits(52..64) {
        0 => true,
        _ => false,
    }
}

#[inline]
const fn canonicalize_physical_address(phys_addr: usize) -> usize {
    phys_addr & 0x000F_FFFF_FFFF_FFFF
}

implement_address!(
    VirtualAddress,
    "virtual",
    "v",
    is_canonical_virtual_address,
    canonicalize_virtual_address,
    page
);



// /// A macro for defining `Page` and `Frame` structs
// /// and implementing their common traits, which are generally identical.
// macro_rules! implement_page_frame {
//     ($TypeName:ident, $desc:literal, $prefix:literal, $address:ident) => {
//         paste! { // using the paste crate's macro for easy concatenation

//             #[doc = "A `" $TypeName "` is a chunk of **" $desc "** memory aligned to a [`PAGE_SIZE`] boundary."]
//             #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
//             pub struct $TypeName {
//                 number: usize,
//             }

//             impl $TypeName {
//                 #[doc = "Returns the `" $address "` at the start of this `" $TypeName "`."]
//                 pub const fn start_address(&self) -> $address {
//                     $address::new_canonical(self.number * PAGE_SIZE)
//                 }

//                 #[doc = "Returns the number of this `" $TypeName "`."]
//                 #[inline(always)]
//                 pub const fn number(&self) -> usize {
//                     self.number
//                 }
                
//                 #[doc = "Returns the `" $TypeName "` containing the given `" $address "`."]
//                 pub const fn containing_address(addr: $address) -> $TypeName {
//                     $TypeName {
//                         number: addr.value() / PAGE_SIZE,
//                     }
//                 }
//             }
//             impl fmt::Debug for $TypeName {
//                 fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//                     write!(f, concat!(stringify!($TypeName), "(", $prefix, "{:#X})"), self.start_address())
//                 }
//             }
//             impl Add<usize> for $TypeName {
//                 type Output = $TypeName;
//                 fn add(self, rhs: usize) -> $TypeName {
//                     // cannot exceed max page number (which is also max frame number)
//                     $TypeName {
//                         number: core::cmp::min(MAX_PAGE_NUMBER, self.number.saturating_add(rhs)),
//                     }
//                 }
//             }
//             impl AddAssign<usize> for $TypeName {
//                 fn add_assign(&mut self, rhs: usize) {
//                     *self = $TypeName {
//                         number: core::cmp::min(MAX_PAGE_NUMBER, self.number.saturating_add(rhs)),
//                     };
//                 }
//             }
//             impl Sub<usize> for $TypeName {
//                 type Output = $TypeName;
//                 fn sub(self, rhs: usize) -> $TypeName {
//                     $TypeName {
//                         number: self.number.saturating_sub(rhs),
//                     }
//                 }
//             }
//             impl SubAssign<usize> for $TypeName {
//                 fn sub_assign(&mut self, rhs: usize) {
//                     *self = $TypeName {
//                         number: self.number.saturating_sub(rhs),
//                     };
//                 }
//             }
//             #[doc = "Implementing `Step` allows `" $TypeName "` to be used in an [`Iterator`]."]
//             impl Step for $TypeName {
//                 #[inline]
//                 fn steps_between(start: &$TypeName, end: &$TypeName) -> Option<usize> {
//                     Step::steps_between(&start.number, &end.number)
//                 }
//                 #[inline]
//                 fn forward_checked(start: $TypeName, count: usize) -> Option<$TypeName> {
//                     Step::forward_checked(start.number, count).map(|n| $TypeName { number: n })
//                 }
//                 #[inline]
//                 fn backward_checked(start: $TypeName, count: usize) -> Option<$TypeName> {
//                     Step::backward_checked(start.number, count).map(|n| $TypeName { number: n })
//                 }
//             }
//         }
//     };
// }

// implement_page_frame!(Page, "virtual", "v", VirtualAddress);
// implement_page_frame!(Frame, "physical", "p", PhysicalAddress);

// // Implement other functions for the `Page` type that aren't relevant for `Frame.
// impl Page {
//     /// Returns the 9-bit part of this `Page`'s [`VirtualAddress`] that is the index into the P4 page table entries list.
//     pub const fn p4_index(&self) -> usize {
//         (self.number >> 27) & 0x1FF
//     }

//     /// Returns the 9-bit part of this `Page`'s [`VirtualAddress`] that is the index into the P3 page table entries list.
//     pub const fn p3_index(&self) -> usize {
//         (self.number >> 18) & 0x1FF
//     }

//     /// Returns the 9-bit part of this `Page`'s [`VirtualAddress`] that is the index into the P2 page table entries list.
//     pub const fn p2_index(&self) -> usize {
//         (self.number >> 9) & 0x1FF
//     }

//     /// Returns the 9-bit part of this `Page`'s [`VirtualAddress`] that is the index into the P1 page table entries list.
//     ///
//     /// Using this returned `usize` value as an index into the P1 entries list will give you the final PTE,
//     /// from which you can extract the mapped [`Frame`]  using `PageTableEntry::pointed_frame()`.
//     pub const fn p1_index(&self) -> usize {
//         (self.number >> 0) & 0x1FF
//     }
// }



// /// A macro for defining `PageRange` and `FrameRange` structs
// /// and implementing their common traits, which are generally identical.
// macro_rules! implement_page_frame_range {
//     ($TypeName:ident, $desc:literal, $short:ident, $chunk:ident, $address:ident) => {
//         paste! { // using the paste crate's macro for easy concatenation
                        
//             #[doc = "A range of [`" $chunk "`]s that are contiguous in " $desc " memory."]
//             #[derive(Clone, PartialEq, Eq)]
//             pub struct $TypeName(RangeInclusive<$chunk>);

//             impl $TypeName {
//                 #[doc = "Creates a new range of [`" $chunk "`]s that spans from `start` to `end`, both inclusive bounds."]
//                 pub const fn new(start: $chunk, end: $chunk) -> $TypeName {
//                     $TypeName(RangeInclusive::new(start, end))
//                 }

//                 #[doc = "Creates a `" $TypeName "` that will always yield `None` when iterated."]
//                 pub const fn empty() -> $TypeName {
//                     $TypeName::new($chunk { number: 1 }, $chunk { number: 0 })
//                 }

//                 #[doc = "A convenience method for creating a new `" $TypeName "` that spans \
//                     all [`" $chunk "`]s from the given [`" $address "`] to an end bound based on the given size."]
//                 pub fn [<from_ $short _addr>](starting_addr: $address, size_in_bytes: usize) -> $TypeName {
//                     assert!(size_in_bytes > 0);
//                     let start = $chunk::containing_address(starting_addr);
//                     // The end bound is inclusive, hence the -1. Parentheses are needed to avoid overflow.
//                     let end = $chunk::containing_address(starting_addr + (size_in_bytes - 1));
//                     $TypeName::new(start, end)
//                 }

//                 #[doc = "Returns the [`" $address "`] of the starting [`" $chunk "`] in this `" $TypeName "`."]
//                 pub const fn start_address(&self) -> $address {
//                     self.0.start().start_address()
//                 }

//                 #[doc = "Returns the number of [`" $chunk "`]s covered by this iterator.\n\n \
//                     Use this instead of [`Iterator::count()`] method. \
//                     This is instant, because it doesn't need to iterate over each entry, unlike normal iterators."]
//                 pub const fn [<size_in_ $chunk:lower s>](&self) -> usize {
//                     // add 1 because it's an inclusive range
//                     (self.0.end().number + 1).saturating_sub(self.0.start().number)
//                 }

//                 /// Returns the size of this range in number of bytes.
//                 pub const fn size_in_bytes(&self) -> usize {
//                     self.[<size_in_ $chunk:lower s>]() * PAGE_SIZE
//                 }

//                 #[doc = "Returns `true` if this `" $TypeName "` contains the given [`" $address "`]."]
//                 pub fn contains_address(&self, addr: $address) -> bool {
//                     self.0.contains(&$chunk::containing_address(addr))
//                 }

//                 #[doc = "Returns the offset of the given [`" $address "`] within this `" $TypeName "`, \
//                     i.e., `addr - self.start_address()`.\n\n \
//                     If the given `addr` is not covered by this range of [`" $chunk "`]s, this returns `None`.\n\n \
//                     # Examples\n \
//                     If the range covers addresses `0x2000` to `0x4000`, then `offset_of_address(0x3500)` would return `Some(0x1500)`."]
//                 pub fn offset_of_address(&self, addr: $address) -> Option<usize> {
//                     if self.contains_address(addr) {
//                         Some(addr.value() - self.start_address().value())
//                     } else {
//                         None
//                     }
//                 }

//                 #[doc = "Returns the [`" $address "`] at the given `offset` into this `" $TypeName "`within this `" $TypeName "`, \
//                     i.e., `addr - self.start_address()`.\n\n \
//                     If the given `offset` is not within this range of [`" $chunk "`]s, this returns `None`.\n\n \
//                     # Examples\n \
//                     If the range covers addresses `0x2000` to `0x4000`, then `address_at_offset(0x1500)` would return `Some(0x3500)`."]
//                 pub fn address_at_offset(&self, offset: usize) -> Option<$address> {
//                     if offset <= self.size_in_bytes() {
//                         Some(self.start_address() + offset)
//                     }
//                     else {
//                         None
//                     }
//                 }

//                 #[doc = "Returns a new separate `" $TypeName "` that is extended to include the given [`" $chunk "`]."]
//                 pub fn to_extended(&self, to_include: $chunk) -> $TypeName {
//                     // if the current range was empty, return a new range containing only the given page/frame
//                     if self.is_empty() {
//                         return $TypeName::new(to_include.clone(), to_include);
//                     }
//                     let start = core::cmp::min(self.0.start(), &to_include);
//                     let end = core::cmp::max(self.0.end(), &to_include);
//                     $TypeName::new(start.clone(), end.clone())
//                 }

//                 #[doc = "Returns an inclusive `" $TypeName "` representing the [`" $chunk "`]s that overlap \
//                     across this `" $TypeName "` and the given other `" $TypeName "`.\n\n \
//                     If there is no overlap between the two ranges, `None` is returned."]
//                 pub fn overlap(&self, other: &$TypeName) -> Option<$TypeName> {
//                     let starts = max(*self.start(), *other.start());
//                     let ends   = min(*self.end(),   *other.end());
//                     if starts <= ends {
//                         Some($TypeName::new(starts, ends))
//                     } else {
//                         None
//                     }
//                 }
//             }
//             impl fmt::Debug for $TypeName {
//                 fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//                     write!(f, "{:?}", self.0)
//                 }
//             }
//             impl Deref for $TypeName {
//                 type Target = RangeInclusive<$chunk>;
//                 fn deref(&self) -> &RangeInclusive<$chunk> {
//                     &self.0
//                 }
//             }
//             impl DerefMut for $TypeName {
//                 fn deref_mut(&mut self) -> &mut RangeInclusive<$chunk> {
//                     &mut self.0
//                 }
//             }
//             impl IntoIterator for $TypeName {
//                 type Item = $chunk;
//                 type IntoIter = RangeInclusive<$chunk>;
//                 fn into_iter(self) -> Self::IntoIter {
//                     self.0
//                 }
//             }
//         }
//     };
// }

// implement_page_frame_range!(PageRange, "virtual", virt, Page, VirtualAddress);
// implement_page_frame_range!(FrameRange, "physical", phys, Frame, PhysicalAddress);

