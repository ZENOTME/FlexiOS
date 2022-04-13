.globl thread_swtch
thread_swtch:
    # source_ctx:x0 , destination_ctx:x1
	stp	x19, x20, [x0, #16 * 0]
	stp	x21, x22, [x0, #16 * 1]
	stp	x23, x24, [x0, #16 * 2]
	stp	x25, x26, [x0, #16 * 3]
	stp	x27, x28, [x0, #16 * 4]
	mov	x28, sp
	stp	x30, x28,  [x0, #16 * 5]

	ldp	x19, x20, [x1, #16 * 0]
	ldp	x21, x22, [x1, #16 * 1]
	ldp	x23, x24, [x1, #16 * 2]
	ldp	x25, x26, [x1, #16 * 3]
	ldp	x27, x28, [x1, #16 * 4]
	ldp	x30, x10,  [x1, #16 * 5]
	mov sp,x10

	ret