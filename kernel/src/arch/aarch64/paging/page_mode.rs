use core::u8;
use crate::{addr_space::PageMode, mm_type::{VirtPageNum, PhysPageNum}};
use super::page_table::*;
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct ARM64;

impl PageMode for ARM64 {
    const FRAME_SIZE_BITS: usize = 12;
    const PPN_BITS: usize = 36;
    const MAX_LEVEL: u8=3;
    fn get_align_for_level(level: u8) ->usize  {
        match level{
            0 => 1,   //4K
            1 => 512 ,//2M
            2 => 512*512,//1G
            3 => 512*512*512,
            _ => unimplemented!("this level does not exist on ARM64")
        }
    }
    
    fn visit_levels_until(level: u8) -> &'static [u8] {
        match level {
            0 => &[3,2,1,0], //usize(2) is 1G 
            1 => &[3,2,1],
            2 => &[3,2],
            3 => &[3],
            _ => unimplemented!("this level does not exist on ARM64"),
        }
    }
    fn visit_page_levels_until(level: u8)-> &'static [u8]{
        match level {
            0 => &[2,1,0], //usize(2) is 1G 
            1 => &[2,1],
            2 => &[2],
            _ => unimplemented!("this level does not exist on ARM64"),
        }
    }

    fn visit_levels_before(level: u8) -> &'static [u8] {
        match level {
            0 => &[3,2,1],
            1 => &[3,2],
            2 => &[3],
            3 => &[],
            _ => unimplemented!("this level does not exist on ARM64"),
        }
    }

    fn visit_levels_from(level: u8) -> &'static [u8] {
        match level {
            0 => &[0],
            1 => &[1,0],
            2 => &[2,1,0],
            3 => &[3,2,1,0],
            _ => unimplemented!("this level does not exist on ARM64"),
        }
    }

    fn vpn_index(vpn: VirtPageNum, level: u8) -> usize {
        (vpn.0 >> (level * 9)) & 511
    }

    fn vpn_level_index(vpn: VirtPageNum, level: u8, idx: usize) -> VirtPageNum {
        VirtPageNum(match level {
            0 => (vpn.0 & !((1 << 9) - 1)) + idx,
            1 => (vpn.0 & !((1 << 18) - 1)) + (idx << 9),
            2 => (vpn.0 & !((1 << 27) - 1)) + (idx << 18),
            3 => (vpn.0 & !((1 << 36) - 1)) + (idx << 27),
            _ => unimplemented!("this level does not exist on ARM64"),
        })
    }

    type PageTable=PageTable;
    type Entry=PageTableEntry;
    type Flags=PageTableFlagsField;

    fn init_page_table(table: &mut Self::PageTable) {
        table.zero();
    }

    fn is_entry_valid(entry: &mut Self::Entry) ->bool {
        entry.is_valid()
    }

    fn set_table(entry: &mut Self::Entry, ppn: PhysPageNum){
        let table_flags=PageTableFlags::VALID::SET+PageTableFlags::TABLE_OR_BLOCK::SET;
        entry.set_frame(ppn, table_flags)
    }

    fn set_frame(entry: &mut Self::Entry, ppn: PhysPageNum,level: u8, flags: Self::Flags){
        let frame_flags=PageTableFlags::VALID::SET+PageTableFlags::TABLE_OR_BLOCK::SET;
        let block_flags = PageTableFlags::VALID::SET;
        match level{
            1|2 =>entry.set_block(ppn, block_flags+flags),
            3 => entry.set_frame(ppn, frame_flags+flags),
            _ => unimplemented!("this level does not exist on ARM64"),
        }
    }
    
    fn set_flags(entry: &mut Self::Entry, flags: Self::Flags) {
        entry.set_flags(flags)
    }

    fn get_ppn(entry: &mut Self::Entry) -> PhysPageNum {
        PhysPageNum(entry.addr().0)
    }
}