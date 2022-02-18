
use core::{cell::{RefCell}, marker::PhantomData, arch, fmt::{Formatter, self,Debug}};

use alloc::vec::Vec;
use zerocopy::FromBytes;

use crate::{addr_type::{VirtAddr, PhysAddr}, frame::{Frame, DataFrame, FrameSize}, arch::paging::{PageTableFlagsField, PageTableFlags,PageTable, page_table}, frame_allocator::{CURRENT_FRAME_ALLOCATOR, UnsafePageAlloctor, FrameAllocator}};

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
        let pg=page_table.as_type_mut::<PageTable>(0).unwrap();
        pg.zero();
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

impl Debug for VmRegion {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("{:?}\nSize:{:?}\n", self.vaddr,self.size))?;
        for _f in self.frames.iter(){
            f.write_fmt(format_args!("Frames: Size: "))?;
            match _f.frame_size(){
                FrameSize::Size4Kb => f.write_fmt(format_args!("4Kb\n"))?,
                FrameSize::Size2Mb => f.write_fmt(format_args!("2Mb\n"))?,
                FrameSize::Size1Gb => f.write_fmt(format_args!("1Gb\n"))?,
            };
            match _f{
                Frame::Data(data) => f.write_fmt(format_args!("{:?}\n",data.frame_addr()))?,
                _=>{}
            };
        }
        f.write_fmt(format_args!("flag: {:#x}\n",self.flag.unwrap().value))?;
        /*
        let exec=PageTableFlags::UXN::SET;
        let wr=PageTableFlags::AP::EL0_RW_ELX_RW;
        let or=PageTableFlags::AP::EL0_OR_ELX_OR;
        let af=PageTableFlags::AF::SET;
        match self.flag{
            
            Some(flag) => {
                if flag.matches_all(exec.value){
                    f.write_fmt(format_args!("NoExec "))?;
                }
                if flag.matches_any(wr.value){
                    f.write_fmt(format_args!("WR "))?;
                }
                if flag.matches_all(or.value){
                    f.write_fmt(format_args!("OR "))?;
                }
                if flag.matches_all(af.value){
                    f.write_fmt(format_args!("ACCESS "))?;
                }
            },
            None => f.write_fmt(format_args!("None"))?,
        }*/
        f.write_fmt(format_args!(""))

    }
}

impl Debug for VmSpace{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        for _r in self.regions.borrow().iter(){
            f.write_fmt(format_args!("===================\n"))?;
            f.write_fmt(format_args!("{:?}\n",_r))?;
        }
        f.write_fmt(format_args!(""))
    }
}