use core::{cell::{RefCell, Cell}};


use crate::{addr_type::*, frame::*};
use alloc::{vec::Vec, collections::VecDeque};
use crate::up::UPSafeCell;
use lazy_static::*;

/// -------------------------
/// frame allocator interface
/// -------------------------
pub trait FrameAllocator{
    fn create_allocator(baddr:PhysAddr,eaddr:PhysAddr)->Self;
    fn allocate_single_frame(& self, size:FrameSize) -> Result<DataFrame,FrameAllocError>;
    fn allocate_frames<P:Addr>(& self,va:P,size:u64) -> Result<Vec<DataFrame>, FrameAllocError>;
    fn deallocate_frame(& self, pf: &DataFrame);
}

pub trait UnsafePageAlloctor{
    fn unsafe_alloc_page(&self)->Result<PhysAddr,FrameAllocError>;
    fn unsafe_deallo(&self,pa: PhysAddr);
}

#[derive(core::fmt::Debug)]
pub enum FrameAllocError{
    AlignedError,
    CapNotEnoughError,
    UnsupportError,
}


type CurrentFrameAllocatorType=StackFrameAllocator;

lazy_static!{
    pub static ref CURRENT_FRAME_ALLOCATOR : UPSafeCell<CurrentFrameAllocatorType> =unsafe {
        extern "C"{
            fn end();
        }
        // Aligned
        let mut baddr=end as u64-KERNEL_BASE;
        baddr=(baddr+4096*512-1)/(4096*512)*(4096*512); 
        let baddr=PhysAddr::new(baddr);
        let eaddr=PhysAddr::new(MEMORY_END);
        UPSafeCell::new(StackFrameAllocator::create_allocator(baddr,eaddr))
    };
}

/// ------------------------
/// A simple stack allocator
/// recycled
/// 0 4Mb
/// 1 2Mb
/// 2 1Gb
/// -----------------------
#[derive(Debug,Clone)]
pub struct StackFrameAllocator {
    current: Cell<PhysAddr>,
    end: PhysAddr,
    recycled: [RefCell<Vec<PhysAddr>>;3],
}

impl StackFrameAllocator{
    // FOR DEBUG
    pub fn print_state(&self){
        println!("curretn:{:x} end:{:x}",self.current.get().addr(),self.end.addr());
    }
    //Not recycled!
    fn new_single_frame(&self,size:FrameSize)->Result<DataFrame,FrameAllocError>{
        if (self.current.get().addr() & (size as u64 -1)) !=0 {return Err(FrameAllocError::AlignedError);}
        if self.current.get()+size as u64 > self.end {return Err(FrameAllocError::CapNotEnoughError);}
        let pa=self.current.replace(self.current.get()+size as u64);
        return Ok(DataFrame::new(pa,size));
    }
}

impl FrameAllocator for StackFrameAllocator{
    fn create_allocator(baddr:PhysAddr,eaddr:PhysAddr)->Self {
       Self{ 
            current: Cell::new(baddr),
            end: eaddr,
            recycled: Default::default(),
        }
    }
    fn allocate_single_frame(& self, size:FrameSize) -> Result<DataFrame,FrameAllocError>{
        match size{
            FrameSize::Size4Kb=>{
                if let Some(pa) = self.recycled[0].borrow_mut().pop(){
                    return Ok(DataFrame::new(pa,FrameSize::Size4Kb));
                }else{
                    return self.new_single_frame(size);
                }
            }
            FrameSize::Size2Mb|FrameSize::Size1Gb=>{
                return Err(FrameAllocError::UnsupportError);
            }
        }
    }
    fn allocate_frames<P:Addr>(& self,va:P,size:u64) -> Result<Vec<DataFrame>, FrameAllocError> {
        //round up to 4Kb
        let size=(size+4095)/4096;
        
        let mut frames=Vec::new();
        let ans=huge_page_alloc_algroithm(va.num(),self.current.get().num(),size);
        let mut pos=ans.len();
        let mut tmp=Vec::new();
        for _i in (0..ans.len()).rev(){
            let _idx:usize=match ans[_i]{
                FrameSize::Size4Kb => 0,
                FrameSize::Size2Mb => 1,
                FrameSize::Size1Gb => 2,
            };
            if let Some(pa)=self.recycled[_idx].borrow_mut().pop(){
                tmp.push(DataFrame::new(pa,ans[_i]));
                pos=_i;
            }else{
                break;
            }
        }
        for _i in 0..pos {
            let sz=ans[_i];
            match self.new_single_frame(sz){
                Ok(frame)=> {frames.push(frame)},
                Err(error)=>{
                    return Err(error); 
                }
            }
        }
        loop{
            if let Some(frame)=tmp.pop(){
                frames.push(frame);
            }
            else{
                break;
            }
        }
        Ok(frames)
    }

    fn deallocate_frame(& self, df: &DataFrame) {
        match df.frame_size(){
            FrameSize::Size4Kb => {
                self.recycled[0].borrow_mut().push(df.frame_addr())
            },
            FrameSize::Size2Mb => {
                self.recycled[1].borrow_mut().push(df.frame_addr())
            },
            FrameSize::Size1Gb => {
                self.recycled[2].borrow_mut().push(df.frame_addr())
            }
        }
    }
}

// -----------------
// Huge page alloc algorithm 
//
// vn VirAddr.num()
// pn PhyAddr.num()
// n PageSize
// return Vec<FrameSize> Ex:
// |4Kb|2Mb|1Gb|2Mb|4Kb|
//-------------------
fn huge_page_alloc_algroithm(vn:u64,pn:u64,n:u64)->VecDeque<FrameSize>{
    //Hard Code
    let aligns:[u64;3]=[512*512,512,1];
    let mut ans:VecDeque<FrameSize>=VecDeque::new();
    for (pos,align) in aligns.iter().enumerate(){
        if (vn % align) != (pn % align) || n<*align {
            continue;
        }
        let (mut ve_prev,mut vs_prev):(Option<u64>,Option<u64>) = (None,None);
        for j in pos..aligns.len(){
            let align=aligns[j];
            let ve_cur = ((vn+align-1)/align)*align;
            let vs_cur=align*((vn+n)/align);
            let frame=match align{
                1=>{FrameSize::Size4Kb}
                512=>{FrameSize::Size2Mb}
                0x40000=>{FrameSize::Size1Gb}
                _=>{FrameSize::Size4Kb}
            };
            if let (Some(ve_prev),Some(vs_prev)) = (ve_prev,vs_prev){
                let l_nframe=(ve_prev-ve_cur)/align;
                let r_nframe=(vs_cur-vs_prev)/align;
                for _i in 0..l_nframe{
                    ans.push_front(frame);
                }
                for _i in 0..r_nframe{
                    ans.push_back(frame);
                }
            }else{
                let nframe=(vs_cur-ve_cur)/align;
                for _i in 0..nframe{
                    ans.push_back(frame);
                }
            }
            (ve_prev,vs_prev) = (Some(ve_cur),Some(vs_cur));
        }   
        break;
    }   
    ans
}

// Unsafe Operate
impl UnsafePageAlloctor for StackFrameAllocator{
    fn unsafe_alloc_page(&self)->Result<PhysAddr,FrameAllocError> {
        if let Some(pa) = self.recycled[0].borrow_mut().pop(){
            return Ok(pa);
        }else{
            if (self.current.get().addr() & (FrameSize::Size4Kb as u64-1)) !=0 {println!("0x{:x}",self.current.get().addr());return Err(FrameAllocError::AlignedError);}
            if (self.current.get()+FrameSize::Size4Kb as u64) > self.end {return Err(FrameAllocError::CapNotEnoughError);}
            let pa=self.current.replace(self.current.get()+FrameSize::Size4Kb as u64);
            return Ok(pa);
        }
    }

    fn unsafe_deallo(&self,pa: PhysAddr) {
        self.recycled[0].borrow_mut().push(pa);
    }
}