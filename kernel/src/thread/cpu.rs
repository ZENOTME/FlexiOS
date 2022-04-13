use conquer_once::spin::OnceCell;

use crate::{thread::ThreadState, up::UPSafeCell};

use super::Thread;
pub static CURRENT_CPU: OnceCell<UPSafeCell<CPU>> = OnceCell::uninit();
pub struct CPU {
    pub cur_thread: Option<Thread>,
}
impl CPU {
    pub fn new() -> Self {
        Self { cur_thread: None }
    }
    pub fn reset(&mut self) -> Thread {
        assert!(self.cur_thread.is_some());
        let mut t = self.cur_thread.take();
        t.as_mut().unwrap().state = ThreadState::READY;
        self.cur_thread = None;
        t.unwrap()
    }
    pub fn cur_thread(&self) -> &Thread {
        &self.cur_thread.as_ref().unwrap()
    }
}
