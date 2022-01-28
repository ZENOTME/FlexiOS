use alloc::vec::Vec;
use crate::arch::paging::page_mode::{ARM64, self};

use super::{
    page_mode::*,
    frame_alloc::*, address::{VirtAddr, PhysAddr},
};
use core::{ops::{Range, RangeBounds}, iter::Step, marker::PhantomData};

// 表示一个分页系统实现的地址空间
//
// 如果属于直接映射或者线性偏移映射，不应当使用这个结构体，应当使用其它的结构体。
#[derive(Debug)]
pub struct PagedAddrSpace<M: PageMode, A: FrameAllocator= StackFrameAllocator> {
    root_frame: FrameBox<A>,
    frames: Vec<FrameBox<A>>,
    frame_alloc: A,
    page_mode: M,
}

#[inline] unsafe fn unref_ppn_mut<'a, M: PageMode>(ppn: PhysPageNum) -> &'a mut M::PageTable {
    let pa = ppn.addr::<M>();
    &mut *(pa.0 as *mut M::PageTable)
}

#[inline] unsafe fn fill_frame_with_initialized_page_table<A: FrameAllocator, M: PageMode>(b: &mut FrameBox<A>) {
    let a = &mut *(b.phys_page_num().addr::<M>().0 as *mut M::PageTable);
    M::init_page_table(a);
}

impl<M: PageMode, A: FrameAllocator + Clone> PagedAddrSpace<M, A> {
    // 创建一个空的分页地址空间。一定会产生内存的写操作
    pub fn try_new_in(page_mode: M, frame_alloc: A) -> Result<Self, FrameAllocError> {
        // 新建一个根页表要求的页帧
        let mut root_frame = FrameBox::try_new_in(frame_alloc.clone())?;
        // 而后，向帧里填入一个空的根页表 
        unsafe { fill_frame_with_initialized_page_table::<A, M>(&mut root_frame) };
        Ok(Self { root_frame, frames: Vec::new(), frame_alloc, page_mode })
    }
    // 得到根页表的地址
    pub fn root_page_number(&self) -> PhysPageNum {
        self.root_frame.phys_page_num()
    }
    // 设置table。如果寻找的过程中，中间的页表没创建，那么创建它们
    unsafe fn alloc_get_table(&mut self, entry_level: PageLevel, vpn_start: VirtPageNum) -> Result<&mut M::PageTable, FrameAllocError> {
        let mut ppn = self.root_frame.phys_page_num();
        for &level in M::visit_levels_before(entry_level) {
            // println!("[] BEFORE PPN = {:x?}", ppn);
            let page_table = unref_ppn_mut::<M>(ppn);
            let vidx = M::vpn_index(vpn_start, level);
            if M::is_entry_valid(&mut page_table[vidx]) {
                ppn = M::get_ppn(&mut page_table[vidx]);
            }
            else{
                let frame_box = FrameBox::try_new_in(self.frame_alloc.clone())?;
                M::set_table(&mut page_table[vidx], frame_box.phys_page_num());
                // println!("[] Created a new frame box");
                ppn = frame_box.phys_page_num();
                self.frames.push(frame_box);
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
    pub fn allocate_map(&mut self, vpn: VirtPageNum, ppn: PhysPageNum, n: usize, flags: M::Flags) -> Result<(), FrameAllocError> {
        for (page_level, ans_range) in MaperSolver::solve(vpn, ppn, n,self.page_mode) {
            for ans in ans_range.step_by(usize::from(M::get_align_for_level(page_level))){
                let vpn=VirtPageNum::new::<M>(VirtAddr::from(ans.0));
                let ppn=PhysPageNum::new::<M>(PhysAddr::from(ans.0));
                let table = unsafe { self.alloc_get_table(page_level, vpn) }?;
                let idx=M::vpn_index(vpn, page_level);
                M::set_frame(&mut table[idx], ppn, page_level, flags.clone());
            }   
        }
        Ok(())
    }
}

// MaperSolver
// Given vpn,ppn,n
// solve the best map using hage page
// anser format is  "iterator of (PageLevel,Vpn,Ppn)""
// From:https://rustmagazine.github.io/rust_magazine_2021/chapter_5/kernel_huge_page_subsystem.html

//Pair use to contain answer(VirPageNum,PhysPageNum)
#[derive(Clone,PartialOrd,PartialEq,Debug)]
pub struct Pair(pub usize,pub usize);
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
pub struct MaperSolver<M:PageMode=defaultMode> {
    ans_iter: alloc::vec::IntoIter<(PageLevel, Range<Pair>) >,
    mode :M
}

impl<M: PageMode> MaperSolver<M> {
    pub fn solve(vpn: VirtPageNum, ppn: PhysPageNum, n: usize,mode:M ) -> Self {
        let mut ans = Vec::new();
        for &i in M::visit_levels_until(PageLevel::leaf_level()) {
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
        Self { ans_iter: ans.into_iter(), mode }
    }
}

impl<M:PageMode> Iterator for MaperSolver<M> {
    type Item = (PageLevel, Range<Pair>);
    fn next(&mut self) -> Option<Self::Item> {
        self.ans_iter.next()
    }
}


pub fn solve_test(){
    let vpn=VirtPageNum::new::<defaultMode>(VirtAddr::from(0));
    let ppn=PhysPageNum::new::<defaultMode>(PhysAddr::from(0));
    for (page_level, ans_range) in MaperSolver::<defaultMode>::solve(vpn,ppn, 0x3e00,ARM64) {
        for ans in ans_range.step_by(usize::from(defaultMode::get_align_for_level(page_level))){
            println!("{} 0x{:X} 0x{:X}",page_level.0,ans.0,ans.1);
        }   
    }
}
