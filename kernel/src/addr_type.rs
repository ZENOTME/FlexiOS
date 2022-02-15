use core::fmt::{self, Debug, Formatter};
use core::ops::{Add,AddAssign,Sub,SubAssign};
use crate::frame_allocator::{CurrentFrameAllocator, FrameAllocator};
/// Re-export the mm_type const
pub use crate::arch::mm_type::{
    PAGE_SIZE,PAGE_SIZE_BITS,PA_WIDTH,VA_WIDTH,KERNEL_BASE
};
pub use crate::arch::board::MEMORY_END;
pub use crate::arch::paging::PageTableFlags;

// ------------------
// General Addr Trait
//-------------------
#[inline]
pub fn floor(addr:usize,align:usize)->usize{
    debug_assert!(align.is_power_of_two(), "`align` must be a power of two");
    addr & !(align-1)
}
#[inline]
pub fn ceil(addr: usize, align: usize) -> usize {
    debug_assert!(align.is_power_of_two(), "`align` must be a power of two");
    let align_mask = align - 1;
    if addr & align_mask == 0 {
        addr // already aligned
    } else {
        (addr | align_mask) + 1
    }
}
pub trait Addr{
    fn addr(&self)->usize;
    fn new(v:usize)->Self;
    fn floor(&self,align:usize) -> Self where Self: Sized{
        Self::new(floor(self.addr(),align))
    }
    fn ceil(&self,align:usize) -> Self where Self: Sized { 
        Self::new(ceil(self.addr(),align))
    }
    fn offset(&self,align:usize) -> usize { 
        self.addr() & (align - 1) 
    }
    fn num(&self) -> usize { 
        self.addr() >> PAGE_SIZE_BITS
    }
    fn is_aligned(&self,align:usize) -> bool { 
        self.offset(align) == 0 
    }
}


/// ---------------
/// Addr Definitions
/// ---------------
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct PhysAddr(pub usize);

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct VirtAddr(pub usize);

impl Addr for PhysAddr{
    fn new(v:usize)->Self{
        PhysAddr::from(v)    
    }

    fn addr(&self)->usize {
        self.0
    }
}

impl Addr for VirtAddr{
    fn new(v:usize)->Self{
        VirtAddr::from(v)    
    }

    fn addr(&self)->usize {
        self.0
    }
}


impl Debug for VirtAddr {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("VA:{:#x}", self.0))
    }
}

impl Debug for PhysAddr {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("PA:{:#x}", self.0))
    }
}

impl From<usize> for PhysAddr {
    fn from(v: usize) -> Self { Self(v & ( (1 << PA_WIDTH) - 1 )) }
}

impl From<usize> for VirtAddr {
    fn from(v: usize) -> Self { Self(v & ( (1 << VA_WIDTH) - 1 )) }
}

impl From<PhysAddr> for usize {
    fn from(v: PhysAddr) -> Self { v.0 }
}

impl From<VirtAddr> for usize {
    fn from(v: VirtAddr) -> Self { v.0 }
}


impl Add<usize> for VirtAddr {
    type Output = Self;
    fn add(self, rhs: usize) -> VirtAddr {
        VirtAddr::from(self.0 + rhs)
    }
}

impl AddAssign<usize> for VirtAddr {
    fn add_assign(&mut self, rhs: usize) {
        *self = *self + rhs;
    }
}

impl Sub<usize> for VirtAddr {
    type Output = Self;
    fn sub(self, rhs: usize) -> Self::Output {
        VirtAddr::from(self.0.checked_sub(rhs).unwrap())
    }
}

impl SubAssign<usize> for VirtAddr {
    fn sub_assign(&mut self, rhs: usize) {
        *self = *self - rhs;
    }
}


impl Add<usize> for PhysAddr {
    type Output = Self;
    fn add(self, rhs: usize) -> Self::Output {
        PhysAddr::from(self.0 + rhs)
    }
}

impl AddAssign<usize> for PhysAddr {
    fn add_assign(&mut self, rhs: usize) {
        *self = *self + rhs;
    }
}

impl Sub<usize> for PhysAddr {
    type Output = Self;
    fn sub(self, rhs: usize) -> Self::Output {
        PhysAddr::from(self.0.checked_sub(rhs).unwrap())
    }
}

impl SubAssign<usize> for PhysAddr {
    fn sub_assign(&mut self, rhs: usize) {
        *self = *self - rhs;
    }
}

// -----------------
// Transform Function
// -----------------
pub fn phys_to_virt(paddr:PhysAddr)->VirtAddr{
    let t=usize::from(paddr);
    VirtAddr(t.checked_add(KERNEL_BASE).unwrap())
}
pub fn virt_to_phys(vaddr:VirtAddr)->PhysAddr{
    let t=usize::from(vaddr);
    PhysAddr(t.checked_sub(KERNEL_BASE).unwrap())
}

