///
/// Interface of the PageSystem
///
use super::address::*;
use core::ops::Range;
use crate::arch::paging::page_mode::ARM64;
/// PageLevel is the abstration concept of mutilple level page
/// Default level => max_level| max_level-1|..|0
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct PageLevel(pub u8); 

impl PageLevel {
    pub const fn leaf_level() -> Self {
        Self(0)
    }
}
// Default Mode
pub type defaultMode=ARM64;
// PageMode is a abstration concept for the arch page system
pub trait PageMode: Copy {
    // 当前分页模式下，页帧大小的二进制位数。例如，4K页为12位。
    const FRAME_SIZE_BITS: usize;
    // 当前分页模式下，物理页号的位数
    const PPN_BITS: usize;
    const MAX_LEVEL:PageLevel;
    // 得到这一层大页物理地址最低的对齐要求
    fn get_align_for_level(level: PageLevel) -> PageAlign;
    // 得到从高到level的页表等级 (n,..,level)
    fn visit_levels_until(level: PageLevel) -> &'static [PageLevel];
    // 得到从高到(level+1)的页表等级，不包括level (n,...,level+1)
    fn visit_levels_before(level: PageLevel) -> &'static [PageLevel];
    // 得到从(level)到低的页表等级 (level,...,0)
    fn visit_levels_from(level: PageLevel) -> &'static [PageLevel];
    // 得到一个虚拟页号对应等级的索引
    fn vpn_index(vpn: VirtPageNum, level: PageLevel) -> usize;
    // 得到一段虚拟页号对应该等级索引的区间；如果超过此段最大的索引，返回索引的结束值为索引的最大值
    //fn vpn_index_range(vpn_range: Range<VirtPageNum>, level: PageLevel) -> Range<usize>;
    // 得到虚拟页号在当前等级下重新索引得到的页号
    fn vpn_level_index(vpn: VirtPageNum, level: PageLevel, idx: usize) -> VirtPageNum;
    // 当前分页模式下，页表的类型
    type PageTable: core::ops::Index<usize, Output = Self::Entry> + core::ops::IndexMut<usize>;
    // 页式管理模式，页表项类型
    type Entry;
    // 页表项的设置
    type Flags : Clone;
    // 创建页表时，把它的所有条目设置为无效条目
    fn init_page_table(table: &mut Self::PageTable);
    // check is the entry valid
    fn is_entry_valid(entry: &mut Self::Entry)->bool;
    // 建立一个到子页表
    fn set_table(entry: &mut Self::Entry, ppn: PhysPageNum);
    // Build a frame
    fn set_frame(entry: &mut Self::Entry, ppn: PhysPageNum, level: PageLevel,flags: Self::Flags);
    // set flag
    fn set_flags(entry: &mut Self::Entry, flags: Self::Flags);
    // 得到一个页表项目包含的物理页号
    fn get_ppn(entry: &mut Self::Entry) -> PhysPageNum;
}
#[derive(Copy, Clone, PartialEq,PartialOrd, Eq, Debug,Ord)]
pub struct PhysPageNum(pub usize);

impl PhysPageNum {
    pub fn addr<M: PageMode>(&self) -> PhysAddr {
        PhysAddr(self.0 << M::FRAME_SIZE_BITS)
    }
    pub fn new<M: PageMode>(paddr:PhysAddr)->Self{
        PhysPageNum(paddr.0>>M::FRAME_SIZE_BITS)
    }
    pub fn next_page(&self) -> PhysPageNum {
        // PhysPageNum不处理具体架构的PPN_BITS，它的合法性由具体架构保证
        PhysPageNum(self.0.wrapping_add(1))
    }
    pub fn is_within_range(&self, begin: PhysPageNum, end: PhysPageNum) -> bool {
        if begin.0 <= end.0 {
            begin.0 <= self.0 && self.0 < end.0
        } else {
            begin.0 <= self.0 || self.0 < end.0
        }
    }
}

#[derive(Copy, Clone, PartialEq,PartialOrd, Eq, Debug,Ord)]
pub struct VirtPageNum(pub usize);

impl VirtPageNum {
    pub fn addr<M: PageMode>(&self) -> VirtPageNum {
        VirtPageNum(self.0 << M::FRAME_SIZE_BITS)
    }
    pub fn new<M: PageMode>(vaddr:VirtAddr)->Self{
        Self(vaddr.0>>M::FRAME_SIZE_BITS)
    }
}

//align relative to the PageNum
//1 align to 4K
//512 align to 2M
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct PageAlign(usize);
impl From<usize> for PageAlign{
    fn from(v:usize)->Self{
        Self(v)
    }
}
impl From<PageAlign> for usize{
    fn from(v:PageAlign)->Self{v.0}
}

