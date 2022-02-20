use core::{
    fmt,
    ops::{Index,IndexMut},
};

use tock_registers::{register_bitfields,fields::FieldValue};
use zerocopy::FromBytes;

use crate::{addr_type::{PhysAddr, Addr, phys_to_kernel}, frame_allocator::{CURRENT_FRAME_ALLOCATOR, UnsafePageAlloctor}, addr_space::{VmRegion}, frame::{Frame, FrameSize}, println};

// -----------------------
// Page Entry Flags 
// -----------------------

register_bitfields! [
    u64,
    pub PageTableFlags[
        /// identifies whether the descriptor is valid
        VALID   OFFSET(0) NUMBITS(1) [],
        /// the descriptor type
        /// 0, Huge Page
        /// 1, Table/4KB_Page
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


// ----------------------------
// A 64-bit page table entry.
// ----------------------------

/// Output address mask
pub const ADDR_MASK: u64 = 0x0000_ffff_ffff_f000;
pub const PGFLAG_MASK: u64 = 0xffff_0000_0000_0fff;


#[derive(Clone, Copy,FromBytes)]
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
        PhysAddr::new((self.entry & ADDR_MASK) as u64)
    }

    /// Get the page_table
    pub fn get_table(&self) -> Option<PhysAddr> {
        if self.is_table_page() {
            let pa=self.addr();
            Some(pa)
        }else{
            None
        }
    }

    /// Returns whether this entry is mapped to a huge page.
    #[inline]
    pub fn is_huge_page(&self) -> bool {
        self.is_valid()&&!self.is_table_or_page()
    }
    // Returns whether this entry is mapped to a table or a page.
    #[inline]
    pub fn is_table_page(&self) -> bool {
        self.is_valid()&&self.is_table_or_page()
    }

    // Return whether this entry is table or page
    #[inline]
    fn is_table_or_page(&self)->bool{
        PageTableFlags::TABLE_OR_BLOCK::SET.matches_all(self.flags().value)
    }
    
    /// Return whether this entry is valid
    #[inline]
    pub fn is_valid(&self) -> bool{
        PageTableFlags::VALID::SET.matches_all(self.flags().value)
    }

    /// Map the entry to the specified physical address with the specified flags.
    #[inline]
    fn set_ppn(&mut self, pa: PhysAddr, flags: PageTableFlagsField) {
        let t:u64=pa.addr();
        self.entry = (t as u64)|flags.value ;
    }
    pub fn set_table_page(&mut self, pa: PhysAddr, flags: Option<PageTableFlagsField>) {
        let new_flags=if let Some(flags)=flags{
            flags+PageTableFlags::TABLE_OR_BLOCK::SET+PageTableFlags::VALID::SET
        }else{
            PageTableFlags::TABLE_OR_BLOCK::SET+PageTableFlags::VALID::SET
        };
        debug_assert!(new_flags.matches_any(PageTableFlags::TABLE_OR_BLOCK::SET.value));
        self.set_ppn(pa, new_flags);
    }
    pub fn set_huge_page(&mut self,pa:PhysAddr,flags: Option<PageTableFlagsField>){
        let new_flags=if let Some(flags)=flags{
            flags+PageTableFlags::TABLE_OR_BLOCK::CLEAR+PageTableFlags::VALID::SET
        }else{
            PageTableFlags::TABLE_OR_BLOCK::CLEAR+PageTableFlags::VALID::SET
        };
        debug_assert!(!new_flags.matches_any(PageTableFlags::TABLE_OR_BLOCK::SET.value));
        self.set_ppn(pa, new_flags);
    }
    // Set flag
    pub fn set_flags(&mut self,flags:Option<PageTableFlagsField>){
        let flags=if let Some(f)=flags{
            f+PageTableFlags::VALID::SET
        }else{
            PageTableFlags::VALID::SET
        };
        self.entry = (self.entry & !PGFLAG_MASK) | flags.value;
    }
    pub fn clear(&mut self){
        self.entry=0;
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


// ----------
// PageTable 
// ----------

#[repr(align(4096))]
#[repr(C)]
#[derive(FromBytes)]
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


    // Only the root_pagetable can use 
    // Find the entry of va at layer and crate it if doesn't exist
    // 0 (root) | 1 (1G) | 2 (2M) | 3(4KB) 
    fn find_entry<'a>(&'a mut self,va:u64,layer:usize)->Result<&'a mut PageTableEntry,&str> {
        let mut current_table:&mut Self=self;
        for _i in 0..layer {
            if let Some(table) = current_table[pg_index(va, _i)].get_table(){
                current_table=unsafe { &mut *(phys_to_kernel(table).addr() as *mut PageTable) };
            }else{
                let new_page;
                if let Ok(pa)=CURRENT_FRAME_ALLOCATOR.exclusive_access().unsafe_alloc_page(){
                    new_page=pa;
                }else{
                    return Err("PageTable:find_entry:Can't not allocate new page");
                }
                current_table[pg_index(va, _i)].set_table_page(new_page, None);
                let table=current_table[pg_index(va, _i)].get_table().unwrap();
                current_table=unsafe { &mut *(phys_to_kernel(table).addr() as *mut PageTable) };
                current_table.zero();
            }

        }
        Ok(&mut current_table[pg_index(va, layer)])
    }

    //PageTableInterface
    pub fn map(&mut self,region:&VmRegion) {
        let mut va=region.start();
        let flag=region.flag();
        for _e in region.get_frames(){
            let sz=_e.frame_size() as u64;
            let layer=match _e.frame_size(){
                 FrameSize::Size4Kb => 3,
                 FrameSize::Size2Mb => 2,
                 FrameSize::Size1Gb => 1
             }; 
            let entry =self.find_entry(va, layer).unwrap();
            match _e{
                Frame::Data(data_frame) => {
                    let pa=data_frame.frame_addr();
                    if layer==3 {
                        entry.set_table_page(pa, flag);
                    }else{
                        entry.set_huge_page(pa,flag);
                    }
                    va+=sz;
                }
                Frame::Guard(guard_frame) => {
                    entry.set_flags(flag);
                    va+=sz;  
                },
                Frame::Lazy(_) => {
                    entry.set_flags(flag);
                    va+=sz;
                },   
            }
        }
    }

    pub fn unmap<T:Addr>(&mut self,region:&VmRegion) {
        todo!()
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
        for (pos,_e) in self.entries.iter().enumerate(){
            f.write_fmt(format_args!("{}:{:?}\n",pos,_e));
        }
        f.write_fmt(format_args!(""))
    }
}


// inlince way 
#[inline]
fn pg_index(addr:u64,idx:usize)->usize{
    let r=match idx{
        0=>(addr>>12>>9>>9>>9)&0x1ff,
        1=>(addr>>12>>9>>9)&0x1ff,
        2=>(addr>>12>>9)&0x1ff,
        3=>(addr>>12)&0x1ff,
        _other=>0
    };
    r as usize
}

