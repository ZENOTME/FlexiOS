
use core::cell::{RefCell};

use alloc::vec::Vec;

use crate::{addr_type::VirtAddr, frame::Frame, arch::paging::{PageTableFlagsField, PageTable}, frame_allocator::{CURRENT_FRAME_ALLOCATOR, UnsafePageAlloctor}};

pub trait PageTableInterface{
    fn map(&mut self,region:&VmRegion);
    fn unmap(&mut self,region:&VmRegion);
}

pub struct VmSpace<'a, P:PageTableInterface>{
    regions:RefCell<Vec<VmRegion>>,
    page_table:&'a mut P,
}

impl <P:PageTableInterface>VmSpace<'_, P>{
    pub fn new()->Self{
        let pg=CURRENT_FRAME_ALLOCATOR.exclusive_access().unsafe_alloc_page().unwrap();
        let pg=unsafe {
            &mut *(pg.0 as *mut P)
        };
        Self{
            regions:RefCell::new(Vec::new()),
            page_table: pg,
        }
    }
    pub fn map_range(&mut self,va:VirtAddr,len:usize,frames:Vec<Frame>,flag:Option<PageTableFlagsField>){
        let region=VmRegion { vaddr:va, size:len, frames:frames, flag:flag };
        self.page_table.map(&region);
        self.regions.borrow_mut().push(region)
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



