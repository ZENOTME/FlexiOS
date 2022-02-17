use core::{mem};
use alloc::{slice};
use crate::{addr_type::{PhysAddr, phys_to_virt}, frame_allocator::{CURRENT_FRAME_ALLOCATOR, FrameAllocator}};
use zerocopy::FromBytes;

// ------------------------
// General FrameSieze Trait
// ------------------------
#[derive(PartialEq,Eq,PartialOrd,Ord,Clone,Copy)]
pub enum FrameSize{
    Size4Kb=4096,
    Size2Mb=4096*512,
    Size1Gb=4096*512*512
}


// -------------
// General Frame
//--------------
pub enum Frame {
    Data(DataFrame),
    Guard(GuardFrame),
    Lazy(LazyFrame)
}


pub trait FRAME{
    fn frame_size(&self)->FrameSize;
}

impl Frame{
    pub fn frame_size(&self)->FrameSize{
        match self{
            Frame::Data(data)=>data.frame_size(),
            Frame::Guard(guard) => guard.frame_size(),
            Frame::Lazy(lazy) => lazy.frame_size()
        }
    }
}
// ------------
// DataFrame
// ------------
#[derive(PartialEq,Eq,PartialOrd, Ord)]
pub struct DataFrame{
    start:PhysAddr,
    size:FrameSize
}

impl FRAME for DataFrame{
    fn frame_size(&self)->FrameSize{
        self.size
    }
}

impl  DataFrame{
    pub fn new(_start:PhysAddr,_size:FrameSize)->Self{
        Self{
            start:_start,
            size:_size
        }
    }

    pub fn frame_addr(&self)->PhysAddr{
        self.start
    }

    pub fn as_slice<T:FromBytes>(&self)->Result<&[T],&'static str>{
        let size_in_byte=mem::size_of::<T>();
        let frame_size=self.size as usize;

        if size_in_byte>frame_size {
            error!("DataFrame::as_slice: requested type {} with size {} ,which is too large for DataFrame of size{}!",core::any::type_name::<T>(),size_in_byte,frame_size);
            return Err("Type too big to contained in this frame");
        }
        let len=frame_size/frame_size;
        let slc: &[T]=unsafe{
            slice::from_raw_parts(phys_to_virt(self.start).0 as *const T,len)
        };
        Ok(slc)
    }

    pub fn as_slice_mut<T:FromBytes>(&self)->Result<&mut [T],&'static str>{
        let size_in_byte=mem::size_of::<T>();
        let frame_size=self.size as usize;

        if size_in_byte>frame_size {
            error!("DataFrame::as_slice_mut: requested type {} with size {} ,which is too large for DataFrame of size{}!",core::any::type_name::<T>(),size_in_byte,frame_size);
            return Err("Type too big to contained in this frame");
        }
        let len=frame_size/frame_size;
        let slc: &mut [T]=unsafe{
            slice::from_raw_parts_mut(phys_to_virt(self.start).0 as *mut T,len)
        };
        Ok(slc)
    }

    pub fn as_type<T:FromBytes>(&self,offset:usize)->Result<&T,&'static str>{
        let size_in_byte=mem::size_of::<T>();
        let frame_size=self.size as usize;
        
        if size_in_byte+offset>frame_size {
            error!("DataFrame::as_type: requested type {} with size {} at offset {},which is too large for DataFrame of size{}!",core::any::type_name::<T>(),size_in_byte,offset,frame_size);
            return Err("Type too big to contained in this frame");
        }
        let t: &T=unsafe{
            &*((phys_to_virt(self.start).0+offset) as *const T)
        };
        Ok(t)
    }

    pub fn as_type_mut<T:FromBytes>(&self,offset:usize)->Result<&mut T,&'static str>{
        let size_in_byte=mem::size_of::<T>();
        let frame_size=self.size as usize;
        
        if size_in_byte+offset>frame_size {
            error!("DataFrame::as_type_mut: requested type {} with size {} at offset {},which is too large for DataFrame of size{}!",core::any::type_name::<T>(),size_in_byte,offset,frame_size);
            return Err("Type too big to contained in this frame");
        }
        let t: &mut T=unsafe{
            &mut *((phys_to_virt(self.start).0+offset) as *mut T)
        };
        Ok(t)
    }
}

impl Drop for DataFrame{
    fn drop(&mut self){
        CURRENT_FRAME_ALLOCATOR.exclusive_access().deallocate_frame(self);
    }
}

#[derive(PartialEq,Eq,PartialOrd, Ord)]
pub struct GuardFrame{
    pub size:FrameSize
}
impl FRAME for GuardFrame {
    fn frame_size(&self)->FrameSize {
        self.size
    }
}
impl GuardFrame{
    pub  fn new(_size:FrameSize)->Self {
        Self{size:_size}
    }
}

#[derive(PartialEq,Eq,PartialOrd, Ord)]
pub struct LazyFrame{
    pub size:FrameSize
}
impl FRAME for LazyFrame {
    fn frame_size(&self)->FrameSize {
        self.size
    }
}
impl LazyFrame{
    pub fn new(_size:FrameSize)->Self {
        Self{size:_size }
    }
}
