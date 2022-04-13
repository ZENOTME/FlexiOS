use crate::{
    addr_space::VmSpace,
    addr_type::{Addr, KernelAddr, PhysAddr, UserAddr},
    arch::{paging::PageTableFlags, UserCtx},
    frame::{FrameObj, FrameSize, GuardFrame},
    frame_allocator::{FrameAllocator, CURRENT_FRAME_ALLOCATOR},
    loader,
    up::UPSafeCell,
};
use alloc::{sync::Arc, vec::Vec};
use core::borrow::Borrow;
use kernel_stack::KernelStack;
use ruspiro_lock::Spinlock;

mod cpu;
mod kernel_stack;
mod scheduler;
mod thread_ctx;
pub use cpu::CPU;
pub use cpu::CURRENT_CPU;
pub use scheduler::_yield;
pub use scheduler::sched;
pub use scheduler::SimpleScheduler;
pub use scheduler::CURRENT_SCHEDULER;
pub use thread_ctx::thread_swtch;

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub enum ThreadState {
    READY,
    RUNNING,
    WAITING,
    EXITING,
    UNINIT,
}

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub enum ThreadType {
    KERNEL,
    USER,
}
pub struct Thread {
    pub _type: ThreadType,
    pub context: thread_ctx::ThreadCtx,
    pub lock: Spinlock,
    pub chanel: Option<usize>,
    pub state: ThreadState,
    space: Option<Arc<UPSafeCell<VmSpace>>>,
    kernel_stack: Option<KernelStack>,
}

impl Default for Thread {
    fn default() -> Self {
        Thread {
            _type: ThreadType::KERNEL,
            context: thread_ctx::ThreadCtx::new(),
            lock: Spinlock::new(),
            chanel: None,
            state: ThreadState::UNINIT,
            space: None,
            kernel_stack: None,
        }
    }
}

impl Thread {
    //create root kernel per core
    //responsible for scheduler work
    pub fn create_root_kernel_thread() -> Self {
        Self {
            _type: ThreadType::KERNEL,
            context: thread_ctx::ThreadCtx::new(),
            lock: Spinlock::new(),
            chanel: None,
            state: ThreadState::RUNNING,
            space: None,
            kernel_stack: None,
        }
    }
    //create root user thread
    //need to create address space
    pub fn create_root_user_thread(
        elf_data: &[u8],
        bin_name: &str,
        stack_base: UserAddr,
        stack_size: u64,
    ) -> Self {
        println!("Create root user thread {}", bin_name);
        let mut space: VmSpace = VmSpace::new();
        //Init User Stack
        let t = CURRENT_FRAME_ALLOCATOR
            .exclusive_access()
            .allocate_frames(stack_base, stack_size)
            .unwrap();
        let mut stack_frames = Vec::new();

        for _frame in t.into_iter() {
            stack_frames.push(FrameObj::Data(_frame));
        }
        stack_frames.push(FrameObj::Guard(GuardFrame::new(FrameSize::Size4Kb)));
        let _stack_flag = PageTableFlags::ATTR_INDEX.val(0)
            + PageTableFlags::SH::INNERSHARE
            + PageTableFlags::AP::EL0_RW_ELX_RW
            + PageTableFlags::UXN::SET
            + PageTableFlags::PXN::SET
            + PageTableFlags::AF::SET;

        space.map_range(
            stack_base.addr(),
            stack_size,
            stack_frames,
            Some(_stack_flag),
        );

        //Load Binary
        let pc = loader::elf_mapper(elf_data, &mut space);

        //Init Kernel Stack
        let kernel_stack_frame = CURRENT_FRAME_ALLOCATOR
            .exclusive_access()
            .allocate_single_frame(FrameSize::Size4Kb)
            .unwrap();
        let mut kernel_stack = KernelStack::new(kernel_stack_frame);
        let mut user_ctx = UserCtx::new();
        user_ctx.user_init(stack_base + stack_size, pc);
        kernel_stack.push_on(user_ctx);
        let mut thread_ctx = thread_ctx::ThreadCtx::new();
        thread_ctx.set(kernel_stack.sp().addr(), ThreadType::USER, 0);
        //println!("SP:{:?}\n",&kernel_stack.sp());
        //println!("{:?}\n",&space);
        //space.print_page_table();
        Self {
            _type: ThreadType::USER,
            context: thread_ctx,
            lock: ruspiro_lock::Spinlock::new(),
            chanel: None,
            state: ThreadState::READY,
            space: Some(Arc::new(unsafe { UPSafeCell::new(space) })),
            kernel_stack: Some(kernel_stack),
        }
    }
    pub fn get_space(&self) -> &UPSafeCell<VmSpace> {
        self.space.as_ref().unwrap().borrow()
    }
    pub fn get_pagetable(&self) -> PhysAddr {
        self.space
            .as_ref()
            .unwrap()
            .as_ref()
            .exclusive_access()
            .get_pagetable()
    }
    pub fn get_kernel_stack(&self) -> KernelAddr {
        self.kernel_stack.as_ref().unwrap().sp()
    }
}
