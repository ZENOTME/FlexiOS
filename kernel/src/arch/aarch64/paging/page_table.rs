use core::{
    fmt,
    ops::{Index,IndexMut}
};
use tock_registers::{register_bitfields,fields::FieldValue};
use crate::mm::address::*;
use crate::mm::frame::PhysFrame;
use crate::mm::page::PageSize;
use super::page::Size4KiB;

register_bitfields! [
    u64,
    pub PageTableFlags[
        /// identifies whether the descriptor is valid
        VALID   OFFSET(0) NUMBITS(1) [],
        /// the descriptor type
        /// 0, Block
        /// 1, Table/Page
        TABLE_OR_BLOCK   OFFSET(1)  NUMBITS(1)[
            BLOCK=0,
            TABLE=1
        ],
        ATTR_INDEX  OFFSET(2) NUMBITS(3) [],
        /// Non-secure bit
        NS  OFFSET(5) NUMBITS(1) [],
        /// Access permission: accessable at EL0
        AP  OFFSET(6) NUMBITS(2) [
            EL0_UNACCESS_ELX_RW=0b00,
            EL0_RW_ELX_RW=0b01,
            EL0_UNACCESS_ELX_OR=0b10,
            EL0_OR_ELX_OR=0b11
        ],
        SH  OFFSET(8) NUMBITS(2) [
            UNSHARE = 0b00,
            OUTERSHARE = 0b10,
            INNERSHARE = 0b11
        ],
        /// Access flag
        AF OFFSET(10) NUMBITS(1)[],
        /// not global bit
        NG OFFSET(11) NUMBITS(1)[],
        /// Dirty Bit Modifier
        DBM OFFSET(51) NUMBITS(1)[],
        /// A hint bit indicating that the translation table entry is one of a contiguous set or entries
        Contiguous OFFSET(52) NUMBITS(1)[],
        /// Privileged Execute-never
        PXN OFFSET(53) NUMBITS(1)[],
        /// Execute-never/Unprivileged execute-never
        UXN OFFSET(54) NUMBITS(1)[],
        /// Software dirty bit
        DIRTY OFFSET(55) NUMBITS(1)[],          
        /// Software swapped bit
        SWAPPED OFFSET(56) NUMBITS(1)[],
        /// Software writable shared bit for COW
        WRITABLE_SHARED OFFSET(57) NUMBITS(1)[],
        /// Software readonly shared bit for COW
        READONLY_SHARED OFFSET(58) NUMBITS(1)[],
        /// Privileged Execute-never for table descriptors
        PXNTable OFFSET(59) NUMBITS(1)[],
        /// Execute-never/Unprivileged execute-never for table descriptors
        XNTable OFFSET(60) NUMBITS(1)[],        
        /// Access permission: access at EL0 not permitted
        APTable_nEL0 OFFSET(61) NUMBITS(1)[],  
        /// Access permission: read-only
        APTable_RO OFFSET(62) NUMBITS(1)[],     
        /// Non-secure bit
        NSTable OFFSET(63) NUMBITS(1)[],        
    ]
];
pub type PageTableFlagsField=FieldValue<u64,PageTableFlags::Register>;


/// A 64-bit page table entry.

/// Output address mask
pub const ADDR_MASK: u64 = 0x0000_ffff_ffff_f000;
pub const PGFLAG_MASK: u64 = 0xffff_0000_0000_0fff;

pub enum FrameError {
    FrameNotPresent,
    HugeFrame,
}

#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct PageTableEntry {
    entry: u64,
}

impl PageTableEntry {
    /// Creates an unused entry.
    #[inline]
    pub const fn new() -> Self {
        Self { entry: 0 }
    }

    /// Returns whether this entry is zero.
    #[inline]
    pub fn is_unused(&self) -> bool {
        self.entry == 0
    }

    /// Sets this entry to zero.
    #[inline]
    pub fn set_unused(&mut self) {
        self.entry = 0;
    }

    /// Returns the flags of this entry.
    #[inline]
    pub fn flags(&self) -> PageTableFlagsField {
        PageTableFlagsField::new(PGFLAG_MASK,0,self.entry)
    }

    /// Returns the physical address mapped by this entry, might be zero.
    #[inline]
    pub fn addr(&self) -> PhysAddr {
        PhysAddr::from((self.entry & ADDR_MASK) as usize)
    }

    /// Returns whether this entry is mapped to a block.
    #[inline]
    pub fn is_block(&self) -> bool {
        !(self.flags().matches_any(PageTableFlags::TABLE_OR_BLOCK::SET.value))
    }
    /// Return whether this entry is valid
    #[inline]
    pub fn is_valid(&self) -> bool{
        self.flags().matches_any(PageTableFlags::VALID::SET.value)
    }
    
    pub fn frame(&self) -> Result<PhysFrame, FrameError> {
        if !self.is_valid() {
            Err(FrameError::FrameNotPresent)
        } else if self.is_block(){
            Err(FrameError::HugeFrame)
        }else {
            Ok(PhysFrame::containing_address(self.addr()))
        }
    }

    /// Map the entry to the specified physical address with the specified flags.
    pub fn set_addr(&mut self, addr: PhysAddr, flags: PageTableFlagsField) {
        debug_assert!(addr.aligned(Size4KiB::SIZE));
        let t:usize=addr.into();
        self.entry = (t as u64)|flags.value ;
    }
    /// Map the entry to the specified physical frame with the specified flags.
    pub fn set_frame(&mut self, frame: PhysFrame, flags: PageTableFlagsField) {
        // is not a block
        debug_assert!(flags.matches_any(PageTableFlags::TABLE_OR_BLOCK::SET.value));
        self.set_addr(frame.start_address(), flags);
    }
    pub fn set_block<S:PageSize>(&mut self,addr:PhysAddr,flags: PageTableFlagsField){
        // is a block
        debug_assert!(!flags.matches_any(PageTableFlags::TABLE_OR_BLOCK::SET.value));
        self.set_addr(addr, flags);
    }
    pub fn set_flags(&mut self,flags:PageTableFlagsField){
        self.entry = (self.entry & !PGFLAG_MASK) | flags.value;
    }

}

impl fmt::Debug for PageTableEntry {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut f = f.debug_struct("PageTableEntry");
        f.field("value", &self.entry);
        f.field("addr", &self.addr());
        f.field("flags", &self.flags().value);
        f.finish()
    }
}

/// The number of entries in a page table.
const ENTRY_COUNT: usize = 512;

/// Represents a page table.
///
/// Always page-sized.
///
/// This struct implements the `Index` and `IndexMut` traits, so the entries can be accessed
/// through index operations. For example, `page_table[15]` returns the 15th page table entry.
#[repr(align(4096))]
#[repr(C)]
pub struct PageTable {
    entries: [PageTableEntry; ENTRY_COUNT],
}

impl PageTable {
    /// Creates an empty page table.
    pub const fn new() -> Self {
        Self {
            entries: [PageTableEntry::new(); ENTRY_COUNT],
        }
    }

    /// Clears all entries.
    pub fn zero(&mut self) {
        for entry in self.entries.iter_mut() {
            entry.set_unused();
        }
    }

    /// Returns an iterator over the entries of the page table.
    pub fn iter(&self) -> impl Iterator<Item = &PageTableEntry> {
        self.entries.iter()
    }

    /// Returns an iterator that allows modifying the entries of the page table.
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut PageTableEntry> {
        self.entries.iter_mut()
    }
}

impl Index<usize> for PageTable {
    type Output = PageTableEntry;

    fn index(&self, index: usize) -> &Self::Output {
        &self.entries[index]
    }
}

impl IndexMut<usize> for PageTable {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.entries[index]
    }
}

impl fmt::Debug for PageTable {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.entries[..].fmt(f)
    }
}