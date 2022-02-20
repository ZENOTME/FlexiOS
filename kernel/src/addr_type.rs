use core::fmt::{self, Debug, Formatter};
use core::ops::{Add,AddAssign,Sub,SubAssign};

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
pub fn floor(addr:u64,align:u64)->u64{
    debug_assert!(align.is_power_of_two(), "`align` must be a power of two");
    addr & !(align-1)
}
#[inline]
pub fn ceil(addr: u64, align: u64) -> u64 {
    debug_assert!(align.is_power_of_two(), "`align` must be a power of two");
    let align_mask = align - 1;
    if addr & align_mask == 0 {
        addr // already aligned
    } else {
        (addr | align_mask) + 1
    }
}

pub trait Addr{
    fn base()->u64 where Self:Sized;
    fn addr(&self)->u64 where Self:Sized;

    fn offset(&self,align:u64) -> u64 where Self:Sized{ 
        self.addr() & (align - 1) 
    }
    fn num(&self) -> u64 where Self:Sized{ 
        self.addr() >> 12
    }
    fn is_aligned(&self,align:u64) -> bool where Self:Sized{ 
        self.offset(align) == 0 
    }
}
const BASE_CLEAR_MASK:u64=0x0000_ffff_ffff_ffff;



/// ---------------
/// PhyAddr Definitions
/// ---------------
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct PhysAddr(u64);

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct KernelAddr(u64);

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct UserAddr(u64);

//Addr trait
impl Addr for PhysAddr{
    #[inline]
    fn base()->u64{
        0
    }
    #[inline]
    fn addr(&self)->u64 {
        self.0
    }
}

impl PhysAddr{
    pub fn new(addr:u64)->Self{
        let addr=addr&BASE_CLEAR_MASK;
        Self(addr|Self::base()) 
    }
    pub fn floor(&self,align:u64) -> Self{
        Self::new(floor(self.addr(),align))
    }
    pub fn ceil(&self,align:u64) -> Self{ 
        Self::new(ceil(self.addr(),align))
    }
}

impl Addr for KernelAddr{
    #[inline]
    fn base()->u64{
        0xffff_0000_0000_0000
    }
    #[inline]
    fn addr(&self)->u64 {
        self.0
    }
}

impl KernelAddr{
    pub fn new(addr:u64)->Self where Self: Sized{
        let addr=addr&BASE_CLEAR_MASK;
        Self(addr|Self::base()) 
    }
    pub fn floor(&self,align:u64) -> Self{
        Self::new(floor(self.addr(),align))
    }
    pub fn ceil(&self,align:u64) -> Self { 
        Self::new(ceil(self.addr(),align))
    }
}

impl Addr for UserAddr{
    #[inline]
    fn base()->u64{
        0
    }
    #[inline]
    fn addr(&self)->u64 {
        self.0
    }
}

impl UserAddr{
    pub fn new(addr:u64)->Self{
        let addr=addr&BASE_CLEAR_MASK;
        Self(addr|Self::base()) 
    }
    pub fn floor(&self,align:u64) -> Self{
        Self::new(floor(self.addr(),align))
    }
    pub fn ceil(&self,align:u64) -> Self{ 
        Self::new(ceil(self.addr(),align))
    }
}

//Debug trait
impl Debug for KernelAddr {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("KA:{:#x}", self.0))
    }
}

impl Debug for UserAddr {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("UA:{:#x}", self.0))
    }
}

impl Debug for PhysAddr {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("PA:{:#x}", self.0))
    }
}

//From trait
impl From<PhysAddr> for u64 {
    fn from(v: PhysAddr) -> Self { v.0 }
}

impl From<UserAddr> for u64 {
    fn from(v: UserAddr) -> Self { v.0 }
}

impl From<KernelAddr> for u64 {
    fn from(v: KernelAddr) -> Self { v.0 }
}

//Op trait
impl Add<u64> for PhysAddr {
    type Output = Self;
    fn add(self, rhs: u64) -> PhysAddr {
        PhysAddr::new(self.0 .checked_add(rhs).unwrap())
    }
}

impl AddAssign<u64> for PhysAddr {
    fn add_assign(&mut self, rhs: u64) {
        *self = *self + rhs;
    }
}

impl Sub<u64> for PhysAddr {
    type Output = Self;
    fn sub(self, rhs: u64) -> Self::Output {
        PhysAddr::new(self.0.checked_sub(rhs).unwrap())
    }
}

impl SubAssign<u64> for PhysAddr {
    fn sub_assign(&mut self, rhs: u64) {
        *self = *self - rhs;
    }
}


impl Add<u64> for UserAddr {
    type Output = Self;
    fn add(self, rhs: u64) -> UserAddr {
        UserAddr::new(self.0 .checked_add(rhs).unwrap())
    }
}

impl AddAssign<u64> for UserAddr {
    fn add_assign(&mut self, rhs: u64) {
        *self = *self + rhs;
    }
}

impl Sub<u64> for UserAddr {
    type Output = Self;
    fn sub(self, rhs: u64) -> Self::Output {
        UserAddr::new(self.0.checked_sub(rhs).unwrap())
    }
}

impl SubAssign<u64> for UserAddr {
    fn sub_assign(&mut self, rhs: u64) {
        *self = *self - rhs;
    }
}
//

impl Add<u64> for KernelAddr {
    type Output = Self;
    fn add(self, rhs: u64) -> KernelAddr {
        KernelAddr::new(self.0 .checked_add(rhs).unwrap())
    }
}

impl AddAssign<u64> for KernelAddr {
    fn add_assign(&mut self, rhs: u64) {
        *self = *self + rhs;
    }
}

impl Sub<u64> for KernelAddr {
    type Output = Self;
    fn sub(self, rhs: u64) -> Self::Output {
        KernelAddr::new(self.0.checked_sub(rhs).unwrap())
    }
}

impl SubAssign<u64> for KernelAddr {
    fn sub_assign(&mut self, rhs: u64) {
        *self = *self - rhs;
    }
}

// -----------------
// Transform Function
// -----------------
pub fn phys_to_kernel(paddr:PhysAddr)->KernelAddr{
    let t=u64::from(paddr);
    KernelAddr::new(t)
}
pub fn kernel_to_phys(kaddr:KernelAddr)->PhysAddr{
    let t=u64::from(kaddr);
    PhysAddr::new(t)
}

