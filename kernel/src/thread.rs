use core::cell::RefCell;

use alloc::{rc::Rc, vec::Vec};
use zerocopy::FromBytes;

use crate::{addr_space::{VmSpace, PageTableInterface}, frame::{DataFrame, FRAME, Frame, GuardFrame, FrameSize}, addr_type::{phys_to_virt, VirtAddr}, frame_allocator::{CURRENT_FRAME_ALLOCATOR, FrameAllocator}, arch::{paging::PageTableFlags, ThreadCtx}, loader};

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
        phys_to_virt(self.data.frame_addr())+self.pos
    }
    pub fn push_on<T>(&mut self,value:T) where T:Sized+FromBytes{
        let ptr=self.data.as_type_mut::<T>(self.pos-core::mem::size_of::<T>() ).unwrap(); 
        *ptr=value;
    }
}

pub struct Thread<'a, P:PageTableInterface>{
    space:Rc<RefCell<VmSpace<'a, P>>>,
    kernel_stack:KernelStack
}

impl <P:PageTableInterface>Thread<'_, P>{
    pub fn create_root_thread(elf_data:&[u8],bin_name:&str,stack_base:VirtAddr,stack_size:usize)->Self{
        let mut space:VmSpace<P>=VmSpace::new();
        //Init User Stack
        let t=CURRENT_FRAME_ALLOCATOR.exclusive_access().allocate_frames(stack_base, stack_size).unwrap();
        let  mut stack_frames=Vec::new();
        for _frame in t.into_iter(){
            stack_frames.push(Frame::Data(_frame));
        }
        stack_frames.push(Frame::Guard(GuardFrame::new(FrameSize::Size4Kb)));
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
            space:Rc::new(RefCell::new(space)),
            kernel_stack: kernel_stack,
        }
    }
}