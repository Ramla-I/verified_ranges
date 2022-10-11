use bit_field::BitField;
use core::{
    cmp::{min, max},
    fmt,
    iter::Step,
    ops::{Add, AddAssign, Deref, DerefMut, RangeInclusive, Sub, SubAssign},
};
use zerocopy::FromBytes;

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
    pub const fn frame_offset(&self) -> usize {
        self.0 & (PAGE_SIZE - 1)
    }
}
impl fmt::Debug for PhysicalAddress {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, concat!("p", "{:#X}"), self.0)
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

///A " "virtual" " memory address, which is a `usize` under the hood.
#[derive(
    Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default, 
    Binary, Octal, LowerHex, UpperHex, 
    BitAnd, BitOr, BitXor, BitAndAssign, BitOrAssign, BitXorAssign, 
    Add, Sub, AddAssign, SubAssign,
    FromBytes,
)]
#[repr(transparent)]
pub struct VirtualAddress(usize);

impl VirtualAddress {
    /// Creates a new `VirtualAddress`, returning an error if the address is not canonical.
    /// This is useful for checking whether an address is valid before using it. 
    /// For example, on x86_64, virtual addresses are canonical
    /// if their upper bits `(64:48]` are sign-extended from bit 47,
    /// and physical addresses are canonical if their upper bits `(64:52]` are 0.
    pub fn new(addr: usize) -> Option<VirtualAddress> {
        if is_canonical_virtual_address(addr) { Some(VirtualAddress(addr)) } else { None }
    }

    ///Creates a new `VirtualAddress` that is guaranteed to be canonical.
    pub const fn new_canonical(addr: usize) -> VirtualAddress {
        VirtualAddress(canonicalize_virtual_address(addr))
    }

    ///Creates a new `VirtualAddress` with a value 0.
    pub const fn zero() -> VirtualAddress {
        VirtualAddress(0)
    }

    ///Returns the underlying `usize` value for this `VirtualAddress`.
    #[inline]
    pub const fn value(&self) -> usize {
        self.0
    }

    /// Returns the offset from the " page " boundary specified by this `VirtualAddress`
    /// For example, if the [`PAGE_SIZE`] is 4096 (4KiB), then this will return
    /// the least significant 12 bits `(12:0]` of this `VirtualAddress`.
    pub const fn page_offset(&self) -> usize {
        self.0 & (PAGE_SIZE - 1)
    }
}
impl fmt::Debug for VirtualAddress {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, concat!("v", "{:#X}"), self.0)
    }
}
impl fmt::Display for VirtualAddress {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}
impl fmt::Pointer for VirtualAddress {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}
impl Add<usize> for VirtualAddress {
    type Output = VirtualAddress;
    fn add(self, rhs: usize) -> VirtualAddress {
        VirtualAddress::new_canonical(self.0.saturating_add(rhs))
    }
}
impl AddAssign<usize> for VirtualAddress {
    fn add_assign(&mut self, rhs: usize) {
        *self = VirtualAddress::new_canonical(self.0.saturating_add(rhs));
    }
}
impl Sub<usize> for VirtualAddress {
    type Output = VirtualAddress;
    fn sub(self, rhs: usize) -> VirtualAddress {
        VirtualAddress::new_canonical(self.0.saturating_sub(rhs))
    }
}
impl SubAssign<usize> for VirtualAddress {
    fn sub_assign(&mut self, rhs: usize) {
        *self = VirtualAddress::new_canonical(self.0.saturating_sub(rhs));
    }
}
impl Into<usize> for VirtualAddress {
    #[inline]
    fn into(self) -> usize {
        self.0
    }
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
