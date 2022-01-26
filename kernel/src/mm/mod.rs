mod heap_allocator;
use heap_allocator::*;
pub use heap_allocator::heap_test;
pub mod address;
pub mod frame_alloc;
pub mod frame;
pub mod page_mode;
//pub mod address;
pub fn init(){
    init_heap();

}