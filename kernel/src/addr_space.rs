use core::iter::Step;
use core::marker::{self, PhantomData};
use core::ops::Range;

use alloc::vec::Vec;
use crate::arch::paging::page_mode::{ARM64};
use crate::frame_allocator::{CurrentFrameAllocator,FrameAllocError, FrameAllocator};
use crate::mm_type::*;


// Default Mode
pub type defaultMode=ARM64;
/// -----------------------------------
/// PageMode is a abstration concept for the arch page system
/// -----------------------------------
pub trait PageMode: Copy {
    // 当前分页模式下，页帧大小的二进制位数。例如，4K页为12位。
    const FRAME_SIZE_BITS: usize;
    // 当前分页模式下，物理页号的位数
    const PPN_BITS: usize;
    const MAX_LEVEL:u8;
    // 得到这一层大页物理地址最低的对齐要求
    fn get_align_for_level(level: u8) -> usize;
    // 得到从高到level的页表等级 (n,..,level)
    fn visit_levels_until(level: u8) -> &'static [u8];
    // get levels (the huge page level)
    fn visit_page_levels_until(level: u8)-> &'static [u8];
    // 得到从高到(level+1)的页表等级，不包括level (n,...,level+1)
    fn visit_levels_before(level: u8) -> &'static [u8];
    // 得到从(level)到低的页表等级 (level,...,0)
    fn visit_levels_from(level: u8) -> &'static [u8];
    // 得到一个虚拟页号对应等级的索引
    fn vpn_index(vpn: VirtPageNum, level: u8) -> usize;
    // 得到一段虚拟页号对应该等级索引的区间；如果超过此段最大的索引，返回索引的结束值为索引的最大值
    //fn vpn_index_range(vpn_range: Range<VirtPageNum>, level: u8) -> Range<usize>;
    // 得到虚拟页号在当前等级下重新索引得到的页号
    fn vpn_level_index(vpn: VirtPageNum, level: u8, idx: usize) -> VirtPageNum;
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
    fn set_frame(entry: &mut Self::Entry, ppn: PhysPageNum, level: u8,flags: Self::Flags);
    // set flag
    fn set_flags(entry: &mut Self::Entry, flags: Self::Flags);
    // 得到一个页表项目包含的物理页号
    fn get_ppn(entry: &mut Self::Entry) -> PhysPageNum;
}

///--------------------------
/// 表示一个分页系统实现的地址空间
///--------------------------
// 如果属于直接映射或者线性偏移映射，不应当使用这个结构体，应当使用其它的结构体。

pub struct PagedAddrSpace<M: PageMode> {
    root_frame: PhysFrame,
    frames: Vec<PhysFrame>,
    regions: Vec<PageRegion<M>>,
    page_mode: marker::PhantomData<M>,
}


struct PageRegion<M:PageMode>{
    pub vpn:VirtPageNum,
    pub npage:usize,
    pub flag: M::Flags,
    pub phy_frames:Vec<PhysFrame>,
    page_mode: marker::PhantomData<M>,
}

#[inline] unsafe fn unref_ppn_mut<'a, M: PageMode>(ppn: PhysPageNum) -> &'a mut M::PageTable {
    let pa = ppn.addr();
    &mut *(pa.0 as *mut M::PageTable)
}

#[inline] unsafe fn fill_frame_with_initialized_page_table<M: PageMode>(b: & PhysFrame) {
    let a = &mut *(b.ppn().addr().0 as *mut M::PageTable);
    M::init_page_table(a);
}

impl<M: PageMode> PagedAddrSpace<M> {
    // 创建一个空的分页地址空间。一定会产生内存的写操作
    pub fn try_new_in(page_mode: M) -> Result<Self, FrameAllocError> {
        // 新建一个根页表要求的页帧
        let mut root_frame = CurrentFrameAllocator.exclusive_access().allocate_frame()?;
        // 而后，向帧里填入一个空的根页表 
        unsafe { fill_frame_with_initialized_page_table::<M>(&root_frame) };
        Ok(Self { root_frame, frames: Vec::new(), regions: Vec::new(),page_mode:PhantomData })
    }
    // 得到根页表的地址
    pub fn root_page_number(&self) -> PhysPageNum {
        self.root_frame.ppn()
    }
    // 设置table。如果寻找的过程中，中间的页表没创建，那么创建它们
    unsafe fn alloc_get_table(&mut self, entry_level: u8, vpn_start: VirtPageNum) -> Result<&mut M::PageTable, FrameAllocError> {
        let mut ppn = self.root_frame.ppn();
        for &level in M::visit_levels_before(entry_level) {
            // println!("[] BEFORE PPN = {:x?}", ppn);
            let page_table = unref_ppn_mut::<M>(ppn);
            let vidx = M::vpn_index(vpn_start, level);
            if M::is_entry_valid(&mut page_table[vidx]) {
                ppn = M::get_ppn(&mut page_table[vidx]);
            }
            else{
                let frame = CurrentFrameAllocator.exclusive_access().allocate_frame()?;
                M::set_table(&mut page_table[vidx], frame.ppn());
                // println!("[] Created a new frame box");
                ppn=frame.ppn();
                self.frames.push(frame);
            }
        }
        // println!("[kernel-alloc-map-test] in alloc_get_table PPN: {:x?}", ppn);
        let page_table = unref_ppn_mut::<M>(ppn); // 此时ppn是当前所需要修改的页表
        // 创建了一个没有约束的生命周期。不过我们可以判断它是合法的，因为它的所有者是Self，在Self的周期内都合法
        Ok(&mut *(page_table as *mut _))
    }
    // set memory map
    // vpn -> ppn
    // size : n
    // flags
    pub fn allocate_map(&mut self, vpn: VirtPageNum, pfs: Vec<PhysFrame>, n: usize, flags: M::Flags) -> Result<(), FrameAllocError> {
        //Map
        let ppn= pfs[0].ppn();
        for (page_level, ans_range) in MaperSolver::<M>::solve(vpn, ppn, n) {
            for ans in ans_range.step_by(usize::from(M::get_align_for_level(page_level))){
                let vpn=VirtPageNum::new(VirtAddr::from(ans.0));
                let ppn=PhysPageNum::new(PhysAddr::from(ans.0));
                let table = unsafe { self.alloc_get_table(page_level, vpn) }?;
                let idx=M::vpn_index(vpn, page_level);
                M::set_frame(&mut table[idx], ppn, page_level, flags.clone());
            }   
        }
        //Set the region
        let region=PageRegion{
            vpn:vpn,
            npage:n,
            flag:flags,
            phy_frames:pfs,
            page_mode:self.page_mode
        };
        self.regions.push(region);
        Ok(())
    }
}

// MaperSolver
// Given vpn,ppn,n
// solve the best map using hage page
// anser format is  "iterator of (u8,Vpn,Ppn)""
// From:https://rustmagazine.github.io/rust_magazine_2021/chapter_5/kernel_huge_page_subsystem.html

//Pair use to contain answer(VirPageNum,PhysPageNum)
#[derive(Clone,PartialOrd,PartialEq,Debug)]
struct Pair(pub usize,pub usize);
impl Step for Pair{
    fn steps_between(start: &Self, end: &Self) -> Option<usize>{
        Some(end.0-start.0)
    }
    fn forward_checked(start: Self, count: usize) -> Option<Self>{
        Some(Self(start.0+count,start.1+count))
    }
    fn backward_checked(start: Self, count: usize) -> Option<Self>{
        Some(Self(start.0+count,start.1+count))
    }
}

#[derive(Debug)]
struct MaperSolver<M:PageMode> {
    ans_iter: alloc::vec::IntoIter<(u8, Range<Pair>) >,
    mode : PhantomData<M>,
}

impl<M: PageMode> MaperSolver<M> {
    pub fn solve(vpn: VirtPageNum, ppn: PhysPageNum, n: usize ) -> Self {
        let mut ans = Vec::new();
        for &i in M::visit_page_levels_until(0) {
            let align = usize::from(M::get_align_for_level(i));
            if (vpn.0 - ppn.0) % align != 0 || n < align {
                continue;
            }
            let (mut ve_prev, mut vs_prev) = (None, None);
            let (mut pe_prev, mut ps_prev) = (None, None);
            for &j in M::visit_levels_from(i) {
                let align_cur = usize::from(M::get_align_for_level(j));
                let ve_cur = align_cur * ((vpn.0 + align_cur - 1) / align_cur); // a * roundup(v / a)
                let vs_cur = align_cur * ((vpn.0 + n) / align_cur); // a * rounddown((v+n) / a)S
                let pe_cur = align_cur * ((ppn.0 + align_cur - 1) / align_cur); // a * roundup(v / a)
                let ps_cur = align_cur * ((ppn.0 + n) / align_cur); // a * rounddown((v+n) / a)
                if let (Some(ve_prev), Some(vs_prev)) = (ve_prev, vs_prev) {
                    let (pe_prev, ps_prev) = (pe_prev.unwrap(), ps_prev.unwrap()); 
                    if ve_cur != ve_prev {
                        ans.push((j, Pair(ve_cur,pe_cur)..Pair(ve_prev,pe_prev)));
                    }
                    if vs_prev != vs_cur {
                        ans.push((j, Pair(vs_prev,ps_prev)..Pair(vs_cur,ps_cur)));
                    }
                } else {
                    if ve_cur != vs_cur { 
                        ans.push((j, Pair(ve_cur,pe_cur)..Pair(vs_cur,ps_cur)));
                    }
                }
                (ve_prev, vs_prev) = (Some(ve_cur), Some(vs_cur));
                (pe_prev, ps_prev) = (Some(pe_cur), Some(ps_cur));
            }
            break;
        } 
        // println!("[SOLVE] Ans = {:x?}", ans);
        Self { ans_iter: ans.into_iter(), mode:PhantomData }
    }
}

impl<M:PageMode> Iterator for MaperSolver<M> {
    type Item = (u8, Range<Pair>);
    fn next(&mut self) -> Option<Self::Item> {
        self.ans_iter.next()
    }
}


pub fn test_solve(){
    let vpn=VirtPageNum::new(VirtAddr::from(0));
    let ppn=PhysPageNum::new(PhysAddr::from(0));
    for (page_level, ans_range) in MaperSolver::<defaultMode>::solve(vpn,ppn, 0x3e00) {
        for ans in ans_range.step_by(usize::from(defaultMode::get_align_for_level(page_level))){
            println!("{} 0x{:X} 0x{:X}",page_level,ans.0,ans.1);
        }   
    }
}
