pub const KERNEL_HEAP_SIZE: usize = 100 * 1024 * 1024;

extern "C" {
    pub fn ekernel();
}