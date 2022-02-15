
use core::cell::{RefCell, Cell};

use alloc::vec::Vec;

use crate::{addr_type::VirtAddr, frame::Frame, arch::paging::PageTableFlagsField};

pub trait PageTable{
    fn map(&mut self,region:&vm_region);
    fn unmap(&mut self,region:&vm_region);
}

struct vm_space<P:PageTable>{
    regions:RefCell<Vec<vm_region>>,
    page_table:P,
}

impl <P:PageTable>vm_space<P>{
    pub fn map_range(&self,va:VirtAddr,len:usize,frames:Vec<Frame>,flag:PageTableFlagsField){
        let region=vm_region { vaddr:va, size:len, frames:frames, flag:flag };
        self.page_table.map(&region);
        self.regions.borrow_mut().push(region)
    }
    pub fn find_region_mut(&self,va:VirtAddr)->&mut vm_region{
        
        todo!()
    }
    pub fn unmap(&self,region:&mut vm_region){
        todo!()
    }
    pub fn remap(&self,region:&mut vm_region){
        todo!()
    }
}

struct vm_region{
    vaddr:VirtAddr,
    size:usize,
    frames:Vec<Frame>,
    flag:PageTableFlagsField
}

impl vm_region{
    pub fn get_frames(&mut self)->&mut Vec<Frame>{
        &mut self.frames
    }
    pub fn replace_flag(&mut self,nf:PageTableFlagsField){
        self.flag=nf;
    }
}



