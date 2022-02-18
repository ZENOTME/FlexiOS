use core::ops::{Index, IndexMut};

use cortex_a::registers::TTBR0_EL1;
use zerocopy::FromBytes;

use crate::{addr_type::{VirtAddr, PhysAddr}, println};

use super::{ eret_to_thread};

const REG_NUM:usize=35;
const SPSR_EL1_EL0t:usize=0b0000;


//----------
//Thread Context
//----------

#[repr(C)]
#[derive(FromBytes)]
pub struct ThreadCtx{
    reg:[u64;REG_NUM]
}

pub enum RegType{
    X0 = 0,			/* 0x00 */
	X1 = 1,			/* 0x08 */
	X2 = 2,			/* 0x10 */
	X3 = 3,			/* 0x18 */
	X4 = 4,			/* 0x20 */
	X5 = 5,			/* 0x28 */
	X6 = 6,			/* 0x30 */
	X7 = 7,			/* 0x38 */
	X8 = 8,			/* 0x40 */
	X9 = 9,			/* 0x48 */
	X10 = 10,		/* 0x50 */
	X11 = 11,		/* 0x58 */
	X12 = 12,		/* 0x60 */
	X13 = 13,		/* 0x68 */
	X14 = 14,		/* 0x70 */
	X15 = 15,		/* 0x78 */
	X16 = 16,		/* 0x80 */
	X17 = 17,		/* 0x88 */
	X18 = 18,		/* 0x90 */
	X19 = 19,		/* 0x98 */
	X20 = 20,		/* 0xa0 */
	X21 = 21,		/* 0xa8 */
	X22 = 22,		/* 0xb0 */
	X23 = 23,		/* 0xb8 */
	X24 = 24,		/* 0xc0 */
	X25 = 25,		/* 0xc8 */
	X26 = 26,		/* 0xd0 */
	X27 = 27,		/* 0xd8 */
	X28 = 28,		/* 0xe0 */
	X29 = 29,		/* 0xe8 */
	X30 = 30,		/* 0xf0 */
	SP_EL0 = 31,		/* 0xf8 */
	ELR_EL1 = 32,		/* 0x100 NEXT PC */
	SPSR_EL1 = 33,		/* 0x108 */
	TPIDR_EL0 = 34
}

impl Index<RegType> for ThreadCtx {
    type Output = u64;
    fn index(&self, index: RegType) -> &Self::Output {
        &self.reg[index as usize]
    }
}

impl IndexMut<RegType> for ThreadCtx {
    fn index_mut(&mut self, index: RegType) -> &mut Self::Output {
        &mut self.reg[index as usize]
    }
}

impl ThreadCtx{
    pub fn new()->Self{
        Self{
            reg: [0;REG_NUM],
        }
    }
    pub fn user_init(&mut self,stack:VirtAddr,pc:VirtAddr){
        self[RegType::SP_EL0]=stack.0 as u64;
        self[RegType::SPSR_EL1]=SPSR_EL1_EL0t as u64;
        self[RegType::ELR_EL1]=pc.0 as u64;
    }
}



pub fn switch_to_vmspace(addr:PhysAddr){
    TTBR0_EL1.set_baddr(addr.0 as u64);
}
pub fn switch_to_context(addr:VirtAddr){
    unsafe{eret_to_thread(addr.0);}
}


