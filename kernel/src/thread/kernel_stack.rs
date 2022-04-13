use crate::{
    addr_type::{phys_to_kernel, KernelAddr},
    frame::DataFrame,
};
use zerocopy::FromBytes;

#[repr(align(4096))]
#[repr(C)]
pub struct KernelStack {
    pos: u64,
    data: DataFrame,
}

impl KernelStack {
    pub fn new(data: DataFrame) -> Self {
        Self {
            pos: data.frame_size() as u64,
            data: data,
        }
    }
    pub fn sp(&self) -> KernelAddr {
        let sp = phys_to_kernel(self.data.frame_addr());
        sp + self.pos
    }
    pub fn push_on<T>(&mut self, value: T)
    where
        T: Sized + FromBytes,
    {
        let ptr = self
            .data
            .as_type_mut::<T>(self.pos - core::mem::size_of::<T>() as u64)
            .unwrap();
        *ptr = value;
        self.pos = self.pos - core::mem::size_of::<T>() as u64;
    }
}
