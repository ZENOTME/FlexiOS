use device_tree::{DeviceTree,Node, util::SliceRead};
use virtio_drivers::{DeviceType, VirtIOHeader, VirtIOBlk};
use core::arch::global_asm;
use alloc::vec;
use crate::arch;

pub mod pl011;
pub mod virtio_impl;
pub mod timer;
pub mod gic;
pub mod console;


pub use pl011::pl01_send;
pub use pl011::pl01_recv;

pub use timer::timer_disable;
pub use timer::timer_enable;

pub use gic::gicv2_disable;
pub use gic::gicv2_enable;


global_asm!(include_str!("dtb.S"));

pub fn driver_init(){
    //init the serail to print
    unsafe{arch::disable_irq();}
    unsafe{pl011::pl01_init();}
    timer::timer_init();
    gic::gicv2_init();
    unsafe{arch::enable_irq();}
    init_dt();
}

//init device tree
fn init_dt(){
    extern "C"{
        fn dtb();
    }
    let dtb=dtb as usize;
    println!("device tree @ {:#x}", dtb);
    #[repr(C)]
    struct DtbHeader {
        be_magic: u32,
        be_size: u32,
    }
    let header = unsafe { &*(dtb as *const DtbHeader) };
    let magic = u32::from_be(header.be_magic);
    const DEVICE_TREE_MAGIC: u32 = 0xd00dfeed;
    println!("device tree magic:{:#x}",magic);
    assert_eq!(magic, DEVICE_TREE_MAGIC);
    let size = u32::from_be(header.be_size);
    println!("device tree size:{:#x}",size);

    let dtb_data = unsafe { core::slice::from_raw_parts(dtb as *const u8, size as usize) };
    let dt = DeviceTree::load(dtb_data).expect("failed to parse device tree");
    walk_dt_node(&dt.root);
}

fn walk_dt_node(dt: &Node) {
    if let Ok(compatible) = dt.prop_str("compatible") {
        if compatible == "virtio,mmio" {
            virtio_probe(dt);
        }
    }
    for child in dt.children.iter() {
        walk_dt_node(child);
    }
}

fn virtio_probe(node: &Node) {
    if let Some(reg) = node.prop_raw("reg") {
        let paddr = reg.as_slice().read_be_u64(0).unwrap();
        let size = reg.as_slice().read_be_u64(8).unwrap();
        let vaddr = paddr+0xffff_0000_0000_0000;
        info!("walk dt addr={:#x}, size={:#x}", paddr, size);
        let header = unsafe { &mut *(vaddr as *mut VirtIOHeader) };
        info!(
            "Detected virtio device with vendor id {:#X}",
            header.vendor_id()
        );
        info!("Device tree node {:?}", node);
        match header.device_type() {
            DeviceType::Block => virtio_blk(header),
            _ => warn!("Unrecognized virtio device"),
        }
    }
}

fn virtio_blk(header: &'static mut VirtIOHeader) {
    let mut blk = VirtIOBlk::new(header).expect("failed to create blk driver");
    let mut input = vec![0xffu8; 512];
    let mut output = vec![0; 512];
    println!("start to test virtio-blk ");
    for i in 0..5 {
        for x in input.iter_mut() {
            *x = i as u8;
        }
        blk.write_block(i, &input).expect("failed to write");
        blk.read_block(i, &mut output).expect("failed to read");
        assert_eq!(input, output);
    }
    info!("virtio-blk test finished");
}