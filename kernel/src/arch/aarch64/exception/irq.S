enable_irq:
	msr	daifclr, #3
	ret

disable_irq:
        msr     daifset, #3
        ret
