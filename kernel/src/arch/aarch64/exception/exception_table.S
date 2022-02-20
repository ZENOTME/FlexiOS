/*===============Const====================
    REG_NUM=35
    SZ_U64=8
    ARCH_EXEC_CONT_SIZE=REG_NUM*SZ_U64=280
    X0 = 0,			X1 = 1,			
	X2 = 2,			X3 = 3,			
	X4 = 4,			X5 = 5,		
	X6 = 6,			X7 = 7,			
	X8 = 8,			X9 = 9,			
	X10 = 10,		X11 = 11,		
	X12 = 12,		X13 = 13,		
	X14 = 14,		X15 = 15,		
	X16 = 16,		X17 = 17,		
	X18 = 18,		X19 = 19,		
	X20 = 20,		X21 = 21,		
	X22 = 22,		X23 = 23,		
	X24 = 24,		X25 = 25,		
	X26 = 26,		X27 = 27,		
	X28 = 28,		X29 = 29,		
	X30 = 30,		SP_EL0 = 31,		
	ELR_EL1 = 32,		SPSR_EL1 = 33,	
	TPIDR_EL0 = 34
*/

//=================================================

.macro handle_entry label
    .align 7
	exception_enter
	mov x0,sp
	bl	\label
	exception_exit
.endm

.macro	exception_enter
	sub	sp, sp, #280
	stp	x0, x1, [sp, #16 * 0]
	stp	x2, x3, [sp, #16 * 1]
	stp	x4, x5, [sp, #16 * 2]
	stp	x6, x7, [sp, #16 * 3]
	stp	x8, x9, [sp, #16 * 4]
	stp	x10, x11, [sp, #16 * 5]
	stp	x12, x13, [sp, #16 * 6]
	stp	x14, x15, [sp, #16 * 7]
	stp	x16, x17, [sp, #16 * 8]
	stp	x18, x19, [sp, #16 * 9]
	stp	x20, x21, [sp, #16 * 10]
	stp	x22, x23, [sp, #16 * 11]
	stp	x24, x25, [sp, #16 * 12]
	stp	x26, x27, [sp, #16 * 13]
	stp	x28, x29, [sp, #16 * 14]
	mrs	x10, sp_el0
	mrs	x11, elr_el1
	mrs	x12, spsr_el1 
	stp	x30, x10, [sp, #16 * 15]
	stp	x11, x12, [sp, #16 * 16]
.endm

.macro	exception_exit
	ldp	x11, x12, [sp, #16 * 16]
	ldp	x30, x10, [sp, #16 * 15] 
	msr	sp_el0, x10
	msr	elr_el1, x11
	msr	spsr_el1, x12
	ldp	x0, x1, [sp, #16 * 0]
	ldp	x2, x3, [sp, #16 * 1]
	ldp	x4, x5, [sp, #16 * 2]
	ldp	x6, x7, [sp, #16 * 3]
	ldp	x8, x9, [sp, #16 * 4]
	ldp	x10, x11, [sp, #16 * 5]
	ldp	x12, x13, [sp, #16 * 6]
	ldp	x14, x15, [sp, #16 * 7]
	ldp	x16, x17, [sp, #16 * 8]
	ldp	x18, x19, [sp, #16 * 9]
	ldp	x20, x21, [sp, #16 * 10]
	ldp	x22, x23, [sp, #16 * 11]
	ldp	x24, x25, [sp, #16 * 12]
	ldp	x26, x27, [sp, #16 * 13]
	ldp	x28, x29, [sp, #16 * 14]
	add	sp, sp, #280
	eret
.endm
/*
 * Vecotr table offsets from vector table base address from ARMv8 Manual
 *	Address		|	Exception Type		| 	Description
 * ============================================================================
 *	VBAR_Eln+0x000	|	 Synchronous		|	 SPSel=0
 * 		+0x080	|	  IRQ/vIRQ		|	Current EL
 *		+0x100	|	  FIQ/vFIQ		|   with Stack Pointer
 * 		+0x180	|	SError/vSError		|    shared with EL0
 * ============================================================================
 *	VBAR_Eln+0x200	|	 Synchronous		|	 SPSel=1
 * 		+0x280	|	  IRQ/vIRQ		|	Current EL
 *		+0x300	|	  FIQ/vFIQ		|   with dedicated
 * 		+0x380	|	SError/vSError		|    Stack Pointer
 * ============================================================================
 *	VBAR_Eln+0x400	|	 Synchronous		|
 * 		+0x480	|	  IRQ/vIRQ		|	Lower EL
 *		+0x500	|	  FIQ/vFIQ		|    using AArch64
 * 		+0x580	|	SError/vSError		|
 * ============================================================================
 *	VBAR_Eln+0x600	|	 Synchronous		|
 * 		+0x680	|	  IRQ/vIRQ		|     	Lower EL
 *		+0x700	|	  FIQ/vFIQ		|    using AArch32
 * 		+0x780	|	SError/vSError		|
 * ============================================================================
 */
.align	11
el1_vector:
 handle_entry syn_exception_handler
 handle_entry exception_handler
 handle_entry exception_handler
 handle_entry exception_handler

 handle_entry syn_exception_handler
 handle_entry exception_handler
 handle_entry exception_handler
 handle_entry exception_handler

 handle_entry syn_exception_handler
 handle_entry exception_handler
 handle_entry exception_handler
 handle_entry exception_handler

 handle_entry syn_exception_handler
 handle_entry exception_handler
 handle_entry exception_handler
 handle_entry exception_handler

set_exception_vector:
    adr	x0, el1_vector
	msr	vbar_el1, x0
	ret

eret_to_thread:
	mov sp,X0
	exception_exit