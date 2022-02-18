
use core::{cell::{RefCell}, marker::PhantomData, arch};

use alloc::vec::Vec;
use zerocopy::FromBytes;

use crate::{addr_type::{VirtAddr, PhysAddr}, frame::{Frame, DataFrame, FrameSize}, arch::paging::{PageTableFlagsField, PageTable, page_table}, frame_allocator::{CURRENT_FRAME_ALLOCATOR, UnsafePageAlloctor, FrameAllocator}};

/*
pub trait PageTableInterface:FromBytes{
    fn map(&mut self,region:&VmRegion);
    fn unmap(&mut self,region:&VmRegion);
}
*/

pub struct VmSpace{
    regions:RefCell<Vec<VmRegion>>,
    page_table:DataFrame
}

impl VmSpace{
    pub fn new()->Self{
        let page_table=CURRENT_FRAME_ALLOCATOR.exclusive_access().allocate_single_frame(FrameSize::Size4Kb).unwrap();
        Self{
            regions:RefCell::new(Vec::new()),
            page_table: page_table,
        }
    }
    pub fn map_range(&mut self,va:VirtAddr,len:usize,frames:Vec<Frame>,flag:Option<PageTableFlagsField>){
        let region=VmRegion { vaddr:va, size:len, frames:frames, flag:flag };
        self.page_table.as_type_mut::<PageTable>(0).unwrap().map(&region);
        self.regions.borrow_mut().push(region)
    }
    pub fn get_pagetable(&self)->PhysAddr{
        self.page_table.frame_addr()
    }
    pub fn find_region_mut(&self,va:VirtAddr)->&mut VmRegion{
        todo!()
    }
    pub fn unmap(&self,region:&mut VmRegion){
        todo!()
    }
    pub fn remap(&self,region:&mut VmRegion){
        todo!()
    }
}

pub struct VmRegion{
    vaddr:VirtAddr,
    size:usize,
    frames:Vec<Frame>,
    flag:Option<PageTableFlagsField>
}

impl VmRegion{
    pub fn get_frames(&self)->& Vec<Frame>{
        &self.frames
    }
    pub fn get_frames_mut(&mut self)->&mut Vec<Frame>{
        &mut self.frames
    }
    pub fn replace_flag(&mut self,nf:PageTableFlagsField){
        self.flag=Some(nf);
    }
    pub fn start(&self)->VirtAddr{
        self.vaddr
    }
    pub fn size(&self)->usize{
        self.size
    }
    pub fn flag(&self)->Option<PageTableFlagsField>{
        self.flag
    }
}



