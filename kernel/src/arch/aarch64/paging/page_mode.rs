use core::u8;

use cortex_a::registers::PAR_EL1::PA;
use tock_registers::fields::Field;

use crate::mm::page_mode::*;
use super::page_table::*;
use core::ops::Range;
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct ARM64;

impl PageMode for ARM64 {
    const FRAME_SIZE_BITS: usize = 12;
    const PPN_BITS: usize = 36;
    const MAX_LEVEL:PageLevel=PageLevel(3);
    fn get_align_for_level(level: PageLevel) -> PageAlign {
        match level.0{
            0 => PageAlign::from(1),   //4K
            1 => PageAlign::from(512), //2M
            2 => PageAlign::from(512*512),//1G
            3 => PageAlign::from(512*512*512),
            _ => unimplemented!("this level does not exist on ARM64")
        }
    }
    
    fn visit_levels_until(level: PageLevel) -> &'static [PageLevel] {
        match level.0 {
            0 => &[PageLevel(2), PageLevel(1), PageLevel(0)], //PageLevel(2) is 1G 
            1 => &[PageLevel(2), PageLevel(1)],
            2 => &[PageLevel(2)],
            _ => unimplemented!("this level does not exist on ARM64"),
        }
    }

    fn visit_levels_before(level: crate::mm::page_mode::PageLevel) -> &'static [crate::mm::page_mode::PageLevel] {
        match level.0 {
            0 => &[PageLevel(3),PageLevel(2), PageLevel(1)],
            1 => &[PageLevel(3),PageLevel(2)],
            2 => &[PageLevel(3)],
            3 => &[],
            _ => unimplemented!("this level does not exist on ARM64"),
        }
    }

    fn visit_levels_from(level: crate::mm::page_mode::PageLevel) -> &'static [crate::mm::page_mode::PageLevel] {
        match level.0 {
            0 => &[PageLevel(0)],
            1 => &[PageLevel(1), PageLevel(0)],
            2 => &[PageLevel(2), PageLevel(1), PageLevel(0)],
            3 => &[PageLevel(3),PageLevel(2), PageLevel(1), PageLevel(0)],
            _ => unimplemented!("this level does not exist on ARM64"),
        }
    }

    fn vpn_index(vpn: VirtPageNum, level: PageLevel) -> usize {
        (vpn.0 >> (level.0 * 9)) & 511
    }
    /*
    fn vpn_index_range(vpn_range: Range<VirtPageNum>, level: PageLevel) -> Range<usize> {
        todo!()
    }*/

    fn vpn_level_index(vpn: crate::mm::page_mode::VirtPageNum, level: crate::mm::page_mode::PageLevel, idx: usize) -> crate::mm::page_mode::VirtPageNum {
        VirtPageNum(match level.0 {
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

    fn set_frame(entry: &mut Self::Entry, ppn: PhysPageNum,level: PageLevel, flags: Self::Flags){
        let frame_flags=PageTableFlags::VALID::SET+PageTableFlags::TABLE_OR_BLOCK::SET;
        let block_flags = PageTableFlags::VALID::SET;
        match level.0{
            1|2 =>entry.set_block(ppn, block_flags+flags),
            3 => entry.set_frame(ppn, frame_flags+flags),
            _ => unimplemented!("this level does not exist on ARM64"),
        }
    }
    
    fn set_flags(entry: &mut Self::Entry, flags: Self::Flags) {
        entry.set_flags(flags)
    }

    fn get_ppn(entry: &mut Self::Entry) -> crate::mm::page_mode::PhysPageNum {
        PhysPageNum::new::<ARM64>(entry.addr())
    }
}