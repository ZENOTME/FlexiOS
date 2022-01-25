use crate::mm::address::*;
pub const PAGE_SIZE:usize=1<<12;
pub const PAGE_SIZE_BITS:usize=12;

pub const VA_WIDTH:usize=48;
pub const PA_WIDTH:usize=48;

pub const KERNEL_BASE:usize=0xffff_0000_0000_0000;
pub fn phys_to_virt(paddr:PhysAddr)->VirtAddr{
    let t=usize::from(paddr);
    VirtAddr(t.checked_add(KERNEL_BASE).unwrap())
}
pub fn virt_to_phys(vaddr:VirtAddr)->PhysAddr{
    let t=usize::from(vaddr);
    PhysAddr(t.checked_sub(KERNEL_BASE).unwrap())
}
