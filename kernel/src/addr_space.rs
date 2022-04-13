use crate::{
    addr_type::PhysAddr,
    arch::paging::{PageTable, PageTableFlagsField},
    frame::{DataFrame, FrameObj, FrameSize},
    frame_allocator::{FrameAllocator, CURRENT_FRAME_ALLOCATOR},
};
use alloc::vec::Vec;
use core::{
    cell::RefCell,
    fmt::{self, Debug, Formatter},
};

/*
pub trait PageTableInterface:FromBytes{
    fn map(&mut self,region:&VmRegion);
    fn unmap(&mut self,region:&VmRegion);
}
*/

pub struct VmSpace {
    regions: RefCell<Vec<VmRegion>>,
    page_table: DataFrame,
}

pub enum AccessSpaceError {
    UnExisted,
    LazyAlloced,
}

impl VmSpace {
    pub fn new() -> Self {
        let page_table = CURRENT_FRAME_ALLOCATOR
            .exclusive_access()
            .allocate_single_frame(FrameSize::Size4Kb)
            .unwrap();
        let pg = page_table.as_type_mut::<PageTable>(0).unwrap();
        pg.zero();
        Self {
            regions: RefCell::new(Vec::new()),
            page_table: page_table,
        }
    }
    pub fn map_range(
        &mut self,
        va: u64,
        len: u64,
        frames: Vec<FrameObj>,
        flag: Option<PageTableFlagsField>,
    ) {
        let region = VmRegion {
            vaddr: va,
            size: len,
            frames: frames,
            flag: flag,
        };
        self.page_table
            .as_type_mut::<PageTable>(0)
            .unwrap()
            .map(&region);
        self.regions.borrow_mut().push(region)
    }
    pub fn get_pagetable(&self) -> PhysAddr {
        self.page_table.frame_addr()
    }
    pub fn print_page_table(&self) {
        let pg = self.page_table.as_type::<PageTable>(0).unwrap();
        println!("Addr:{:?}\n{:?}", self.get_pagetable(), pg);
    }

    pub fn read_from_space(&self, buf: &mut [u8], va: u64) -> Result<usize, AccessSpaceError> {
        for _region in self.regions.borrow().iter() {
            if _region.is_in_range(va) {
                let mut off = va - _region.start();
                let mut len = buf.len() as u64;
                let mut pos = 0;
                for _frame in _region.frames.iter() {
                    let frame_size = _frame.frame_size() as u64;
                    if len > 0 && off < frame_size {
                        off = 0;
                        match _frame {
                            FrameObj::Data(data) => {
                                let l = if (frame_size - off) < len {
                                    frame_size - off
                                } else {
                                    len
                                };
                                let src = data.as_slice::<u8>(off, l).unwrap();
                                let dst = &mut buf[pos as usize..(pos + l) as usize];
                                dst.copy_from_slice(src);
                                pos += l;
                                len -= l;
                            }
                            _ => {
                                return Err(AccessSpaceError::UnExisted);
                            }
                        }
                    } else {
                        off = off - frame_size;
                    }
                }
            }
        }
        Ok(0)
    }

    pub fn find_region_mut(&self, va: u64) -> &mut VmRegion {
        todo!()
    }
    pub fn unmap(&self, region: &mut VmRegion) {
        todo!()
    }
    pub fn remap(&self, region: &mut VmRegion) {
        todo!()
    }
}

pub struct VmRegion {
    vaddr: u64,
    size: u64,
    frames: Vec<FrameObj>,
    flag: Option<PageTableFlagsField>,
}

impl VmRegion {
    pub fn get_frames(&self) -> &Vec<FrameObj> {
        &self.frames
    }
    pub fn get_frames_mut(&mut self) -> &mut Vec<FrameObj> {
        &mut self.frames
    }
    pub fn replace_flag(&mut self, nf: PageTableFlagsField) {
        self.flag = Some(nf);
    }
    pub fn start(&self) -> u64 {
        self.vaddr
    }
    pub fn size(&self) -> u64 {
        self.size
    }
    pub fn flag(&self) -> Option<PageTableFlagsField> {
        self.flag
    }
    pub fn is_in_range(&self, va: u64) -> bool {
        let b = self.vaddr;
        let e = self.vaddr + self.size;
        va >= b && va < e
    }
}

impl Debug for VmRegion {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("VA:{:#x}\nSize:{:?}\n", self.vaddr, self.size))?;
        for _f in self.frames.iter() {
            f.write_fmt(format_args!("Frames: Size: "))?;
            match _f.frame_size() {
                FrameSize::Size4Kb => f.write_fmt(format_args!("4Kb\n"))?,
                FrameSize::Size2Mb => f.write_fmt(format_args!("2Mb\n"))?,
                FrameSize::Size1Gb => f.write_fmt(format_args!("1Gb\n"))?,
            };
            match _f {
                FrameObj::Data(data) => f.write_fmt(format_args!("{:?}\n", data.frame_addr()))?,
                _ => {}
            };
        }
        f.write_fmt(format_args!("flag: {:#x}\n", self.flag.unwrap().value))?;
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

impl Debug for VmSpace {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        for _r in self.regions.borrow().iter() {
            f.write_fmt(format_args!("===================\n"))?;
            f.write_fmt(format_args!("{:?}\n", _r))?;
        }
        f.write_fmt(format_args!(""))
    }
}
