use core::{mem};
use alloc::{slice};
use crate::{addr_type::{PhysAddr, phys_to_virt}, frame_allocator::{CurrentFrameAllocator, FrameAllocator}};
use zerocopy::FromBytes;

// ------------------------
// General FrameSieze Trait
// ------------------------

pub enum FrameSize{
    Size4Kb=4096,
    Size2Mb=4096*512,
    Size1Gb=4096*512*512
}


// -------------
// General Frame
//--------------
pub enum Frame {
    Data(DataFrame)
}

impl Drop for Frame{
    fn drop(&mut self){
        match self{
            Frame::Data(data) => {
                CurrentFrameAllocator.exclusive_access().deallocate_frame(data);
            },
        }
    }
}


// ------------
// DataFrame
// ------------
pub struct DataFrame{
    start:PhysAddr,
    size:FrameSize
}

impl DataFrame{
    pub fn frame_addr(&self)->PhysAddr{
        self.start
    }
    pub fn frame_size(&self)->FrameSize{
        self.size
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

