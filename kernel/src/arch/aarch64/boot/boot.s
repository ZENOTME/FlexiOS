.section .text.boot
.globl _start

_start:
    # pc == 0x80000
    # only the primary CPU starts here, other CPUs start from 0x0 and spin
    # until the spin table holds a jump address. (see qemu/hw/arm/raspi.c)

    # read cpu affinity, start core 0, halt rest
    mrs     x19, mpidr_el1
    and     x19, x19, #3
    cbnz    x19, halt
    bl      el_setup
    bl      clear_bss
    bl      create_init_paging
    b       master_startup

halt:
    # unreachable
    wfe
    b       halt

# switch to EL1, setup system registers of other EL
# and set the temporary stack for low address 
el_setup:
    # use SP_ELx for Exception level ELx
    msr     SPsel, #1

    # read the current exception level into x0 (ref: C5.2.1)
    mrs     x0, CurrentEL
    and     x0, x0, #0b1100
    lsr     x0, x0, #2
switch_to_el2:
    # switch to EL2 if we're in EL3. otherwise switch to EL1
    cmp     x0, #2
    beq     switch_to_el1
    cmp     x0, #1
    beq     el_setup_end

    # at EL3
	# 0 non-secure state
	# 4 5 RESET1
	# 7 DISABLE SECURE INS
	# 8 DISABLE HYPERVISIOR CALL
	# 10 SET EL2 EL1 TO AARCH64  
    # set-up SCR_EL3 (bits 0, 4, 5, 7, 8, 10) (A53: 4.3.42)
    mov     x0, #0x5b1
    msr     scr_el3, x0

	# 0 AARCH64
	# 3 RETURN TO EL2
	# 6789 DISABLE DAIF
    # set-up SPSR_EL3 (bits 0, 3, 6, 7, 8, 9) (ref: C5.2.20)
    mov     x0, #0x3c9
    msr     spsr_el3, x0

    # switch
    adr     x0, switch_to_el1
    msr     elr_el3, x0
    eret
switch_to_el1:
    # switch to EL1 if we're not already in EL1. otherwise continue with start
    cmp     x0, #1
    beq     el_setup_end

    # at EL2
    # set the temporary stack for EL1 in lower VA range
    adrp    x0, _start
    sub     x0, x0, x19, lsl #16
    msr     sp_el1, x0

    # set-up HCR_EL2, enable AArch64 in EL1 (bits 1, 31) (ref: D10.2.45)
    mov     x0, #0x0002
    movk    x0, #0x8000, lsl #16
    msr     hcr_el2, x0

    # don't trap accessing SVE registers (ref: D10.2.30)
    msr     cptr_el2, xzr

    # enable floating point and SVE (SIMD) (bits 20, 21) (ref: D10.2.29)
    mrs     x0, cpacr_el1
    orr     x0, x0, #(0x3 << 20)
    msr     cpacr_el1, x0

    # Set SCTLR to known state (RES1: 11, 20, 22, 23, 28, 29) (ref: D10.2.100)
    mov     x0, #0x0800
    movk    x0, #0x30d0, lsl #16
    msr     sctlr_el1, x0

    # set-up SPSR_EL2 (bits 0, 2, 6, 7, 8, 9) (ref: C5.2.19)
    mov     x0, #0x3c5
    msr     spsr_el2, x0

    # DIABLE EL1 TIMER TRAPS
    # NOTE: This doesn't actually enable the counter stream.
    mrs     x0, cnthctl_el2
    orr     x0, x0, #3
    msr     cnthctl_el2, x0
    msr     cntvoff_el2, xzr

    # switch
	# lr is the link register stores the return address
    msr     elr_el2, lr
    eret
el_setup_end:
    # at EL1
    adrp    x0, _start
    sub     x0, x0, x19, lsl #16
    mov     sp, x0
    ret

# primary CPU: enable paging, jump to upper VA range
master_startup:
    bl      invalidate_cache_all
    bl      enable_mmu
    ldr     x0, =master_main
    b       jump_to_main

# other CPUs: jump to EL1, enable paging, jump to upper VA range
.global slave_startup
slave_startup:
    mrs     x19, mpidr_el1
    and     x19, x19, #3
    bl      el_setup
    #bl      enable_mmu
    #ldr     x0, =others_main
    b      jump_to_main

# set-up kernel stack, jump to master_main/others_main
jump_to_main:
    # msr     ttbr0_el1, xzr
    ldr     x8, =bootstacktop
    sub     x8, x8, x19, lsl #18
    mov     sp, x8
    mov     x29, xzr
    mov     x30, xzr
    br      x0

# ...
invalidate_cache_all:
	mrs     x0, clidr_el1
	and     w3, w0, #0x07000000     // get 2x level of coherence
	lsr     w3, w3, #23
	cbz     w3, .Lfinished_inv_cache
	mov     w10, #0                 // w10 = 2x cache level
	mov     w8, #1                  // w8 = constant 1
.Lloop1_inv_cache:
	add     w2, w10, w10, lsr #1    // calculate 3x cache level
	lsr     w1, w0, w2              // extract 3 bit cache type for this level
	and     w1, w1, #0x7
	cmp     w1, #2
	b.lt    .Lskip_inv_cache            // no data or unified cache at this level
	msr     csselr_el1, x10         // select this cache level
	isb                             // synchronize change to csselr
	mrs     x1, ccsidr_el1          // w1 = ccsidr
	and     w2, w1, #7              // w2 = log2(line len) - 4
	add     w2, w2, #4              // w2 = log2(line len)
	ubfx    w4, w1, #3, #10         // w4 = max way number, right aligned
	clz     w5, w4                  // w5 = 32 - log2(ways), bit position of way in DC operand
	lsl     w9, w4, w5              // w9 = max way number, aligned to position in DC operand
	lsl     w12, w8, w5             // w12 = amount to decrement way number per iteration

.Lloop2_inv_cache:
	ubfx    w7, w1, #13, #15        // w7 = max set number, right aligned
	lsl     w7, w7, w2              // w7 = max set number, aligned to position in DC operand
	lsl     w13, w8, w2             // w13 = amount to decrement set number per iteration
.Lloop3_inv_cache:
	orr     w11, w10, w9            // w11 = combine way number and cache number
	orr     w11, w11, w7            //       and set number for DC operand
	dc      isw, x11                // data cache op
	subs    w7, w7, w13             // decrement set number
	b.ge    .Lloop3_inv_cache

	subs    w9, w9, w12             // decrement way number
	b.ge    .Lloop2_inv_cache
.Lskip_inv_cache:
	add     w10, w10, #2            // increment 2x cache level
	cmp     w3, w10
	dsb     sy                      // ensure completetion of previous cache maintainance instructions
	b.gt    .Lloop1_inv_cache
.Lfinished_inv_cache:

	// dump the instruction cache as well
	ic      iallu
	isb
	ret


.section .bss.stack
.align 12
.global bootstack
.global bootstacktop
bootstack:
    .space 0x100000 // 1M
bootstacktop:


.section .data
.align 12
page_table_lvl0:
    .space 0x1000 // 4K
page_table_lvl1:
    .space 0x1000 // 4K
page_table_lvl2:
    .space 0x1000 // 4K