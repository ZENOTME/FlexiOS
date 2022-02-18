use core::{cell::RefCell};


use alloc::{ vec::Vec, sync::Arc};
use zerocopy::FromBytes;

use crate::{addr_space::{VmSpace}, frame::{DataFrame, FRAME, Frame, GuardFrame, FrameSize}, addr_type::{phys_to_virt, VirtAddr, PhysAddr}, frame_allocator::{CURRENT_FRAME_ALLOCATOR, FrameAllocator}, arch::{paging::PageTableFlags, ThreadCtx}, loader, scheduler::ThreadState, up::UPSafeCell};

const KERNEL_STACK_SIZE:usize=4096;

#[repr(align(4096))]
#[repr(C)]
struct KernelStack{
    pos:usize,
    data:DataFrame
}

impl KernelStack{
    pub fn new(data:DataFrame)->Self{
        Self{
            pos:data.frame_size() as usize,
            data:data
        }
    }
    pub fn sp(&self)->VirtAddr{
        let sp=phys_to_virt(self.data.frame_addr());
        let sp=sp.0+self.pos;
        let sp=VirtAddr::from(sp);
        sp
        
    }
    pub fn push_on<T>(&mut self,value:T) where T:Sized+FromBytes{
        let ptr=self.data.as_type_mut::<T>(self.pos-core::mem::size_of::<T>() ).unwrap(); 
        *ptr=value;
    }
}

pub struct Thread{
    state:ThreadState,
    space:Arc<UPSafeCell<VmSpace>>,
    kernel_stack:KernelStack
}

impl Thread{
    pub fn create_root_thread(elf_data:&[u8],bin_name:&str,stack_base:VirtAddr,stack_size:usize)->Self{
        let mut space:VmSpace=VmSpace::new();
        //Init User Stack
        let t=CURRENT_FRAME_ALLOCATOR.exclusive_access().allocate_frames(stack_base, stack_size).unwrap();
        let  mut stack_frames=Vec::new();
        for _frame in t.into_iter(){
            stack_frames.push(Frame::Data(_frame));
        }
        stack_frames.push(Frame::Guard(GuardFrame::new(FrameSize::Size4Kb)));
        println!("stack page num {}",stack_frames.len());
        let stack_flag=PageTableFlags::ATTR_INDEX.val(0)+PageTableFlags::SH::INNERSHARE+PageTableFlags::AP::EL0_RW_ELX_RW+PageTableFlags::UXN::SET+PageTableFlags::PXN::SET;
        space.map_range(stack_base, stack_size, stack_frames, Some(stack_flag));
        //Load Binary
        let pc=loader::elf_mapper(elf_data,&mut space);
        //Init Kernel Stack
        let kernel_stack_frame=CURRENT_FRAME_ALLOCATOR.exclusive_access().allocate_single_frame(FrameSize::Size4Kb).unwrap();
        let mut kernel_stack=KernelStack::new(kernel_stack_frame);
        let mut thread_ctx=ThreadCtx::new();
        thread_ctx.user_init(stack_base+stack_size, pc);
        kernel_stack.push_on(thread_ctx);
        Self{
            state:ThreadState::TS_READY,
            space:Arc::new(unsafe{UPSafeCell::new(space)}),
            kernel_stack: kernel_stack,
        }
    }
    pub fn get_pagetable(&self)->PhysAddr{
        self.space.as_ref().exclusive_access().get_pagetable()
    }
    pub fn get_kernel_stack(&self)->VirtAddr{
        self.kernel_stack.sp()

    }
    pub fn get_state(&self)->ThreadState{
        self.state
    }
    pub fn set_state(&mut self,state:ThreadState){
        self.state=state;
    }
}