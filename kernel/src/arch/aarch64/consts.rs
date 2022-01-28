pub const KERNEL_HEAP_SIZE: usize = 8 * 1024 * 1024;
pub const FRAME_BIT:usize = 12;
extern "C" {
    pub fn ekernel();
}