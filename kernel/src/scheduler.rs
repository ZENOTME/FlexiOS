use core::cell::RefCell;

use alloc::collections::VecDeque;

use crate::{thread::Thread, arch::{ switch_to_vmspace, switch_to_context}, up::UPSafeCell, addr_type::{Addr, KernelAddr}};
use lazy_static::*;
#[derive(Clone,Copy,PartialEq,Eq)]
pub enum ThreadState{
    TS_READY=0,
    TS_RUNNING=1,
    TS_WAITING=2,
    TS_EXITING=3
}

pub trait Scheduler{
    fn push_thread(&mut self,t:Thread);
    fn sched(&mut self)->Result<KernelAddr,SchedFail>;
}

#[derive(core::fmt::Debug)]
pub struct SchedFail;

type CURRENT_TYPE=SimpleScduler;
lazy_static!{
    pub static ref CURRENT_SCHEDULER : UPSafeCell<CURRENT_TYPE> =unsafe {
        UPSafeCell::new(CURRENT_TYPE::new())
    };
}


pub struct SimpleScduler{
    queue:VecDeque<Thread>,
    cur:Option<usize>
}
impl SimpleScduler{
    pub fn new()->Self{
        Self{
            queue:VecDeque::new(),
            cur:None
        }
    }
    pub fn cur_thread(&mut self)->Option<&mut Thread>{
        match self.cur{
            None=>None,
            Some(pos)=>{
                Some(&mut self.queue[pos])
            }
        }
    }
}

impl Scheduler for SimpleScduler{
    fn push_thread(&mut self,t:Thread) {
        self.queue.push_back(t);
    }

    fn sched(&mut self) ->Result<KernelAddr,SchedFail>{
        if let Some(cur)=self.cur{
            match self.queue[cur].get_state(){
                ThreadState::TS_RUNNING=>self.queue[cur].set_state(ThreadState::TS_READY),
                other=>{return Err(SchedFail);}
            }
            for (pos,_e) in self.queue.iter().enumerate(){
                if cur!=pos && _e.get_state()==ThreadState::TS_READY{
                    switch_to_vmspace(_e.get_pagetable());
                    self.cur=Some(pos);
                    return Ok(_e.get_kernel_stack());
                }
            }
        }else{ 
            for (pos,_e) in self.queue.iter().enumerate(){
                if _e.get_state()==ThreadState::TS_READY{
                    switch_to_vmspace(_e.get_pagetable());
                    self.cur=Some(pos);
                    return Ok(_e.get_kernel_stack());
                }
            }
        }
        Err(SchedFail)
    }
}