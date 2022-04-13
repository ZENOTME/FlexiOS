use super::{cpu::CURRENT_CPU, thread_ctx::ThreadCtx, thread_swtch, ThreadState};
use crate::{arch::switch_to_vmspace, thread::Thread, up::UPSafeCell};
use alloc::collections::VecDeque;
use conquer_once::spin::OnceCell;

//---------- Scheduler Trait -----------
#[derive(core::fmt::Debug)]
pub struct SchedFail;

//---------------------------------------
pub static CURRENT_SCHEDULER: OnceCell<UPSafeCell<SimpleScheduler>> = OnceCell::uninit();

pub struct SimpleScheduler {
    queue: VecDeque<Thread>,
    sched_thread: Thread,
}

impl SimpleScheduler {
    pub fn sched_context(&self) -> &ThreadCtx {
        &self.sched_thread.context
    }

    pub fn new() -> Self {
        Self {
            queue: VecDeque::new(),
            sched_thread: Thread::create_root_kernel_thread(),
        }
    }

    pub fn push_thread(&mut self, t: Thread) {
        self.queue.push_back(t);
    }

    pub fn sched_next(&mut self) -> Option<Thread> {
        let mut cur: Option<usize> = None;
        for (_pos, _t) in self.queue.iter_mut().enumerate() {
            if _t.state == ThreadState::READY {
                cur = Some(_pos);
                _t.state = ThreadState::RUNNING;
                break;
            }
        }
        if let Some(pos) = cur {
            return self.queue.remove(pos);
        }
        None
    }
}

pub fn sched() {
    let scheduler_context = CURRENT_SCHEDULER
        .try_get()
        .expect("No init")
        .exclusive_access()
        .sched_context()
        .get_raw_addr();
    loop {
        let t = CURRENT_SCHEDULER
            .try_get()
            .expect("No init")
            .exclusive_access()
            .sched_next()
            .unwrap();
        let t_context = t.context.get_raw_addr();
        switch_to_vmspace(t.get_pagetable());
        CURRENT_CPU
            .try_get()
            .expect("No init")
            .exclusive_access()
            .cur_thread = Some(t);

        unsafe {
            thread_swtch(scheduler_context, t_context);
        }
    }
}

pub fn _yield() {
    let scheduler_context = CURRENT_SCHEDULER
        .try_get()
        .expect("No init")
        .exclusive_access()
        .sched_context()
        .get_raw_addr();
    let t = CURRENT_CPU
        .try_get()
        .expect("No init")
        .exclusive_access()
        .reset();
    let t_context = t.context.get_raw_addr();
    CURRENT_SCHEDULER
        .try_get()
        .expect("No init")
        .exclusive_access()
        .push_thread(t);
    unsafe {
        thread_swtch(t_context, scheduler_context);
    }
}
