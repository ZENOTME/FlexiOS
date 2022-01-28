mod heap_allocator;
use heap_allocator::*;
pub use heap_allocator::heap_test;
pub mod address;
pub mod frame_alloc;
pub mod frame;
pub mod page_mode;
pub mod page_space;
pub use page_space::*;
//pub mod address;
pub fn init(){
    init_heap();

}