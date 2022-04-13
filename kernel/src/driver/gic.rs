use crate::{
    addr_type::KernelAddr,
    thread::{self, _yield},
};

const GICD_BASE: usize = 0xffff_0000_0800_0000;
const GICC_BASE: usize = 0xffff_0000_0801_0000;

/* Distributor */
const GICD_CTLR: usize = GICD_BASE + 0x000;
const GICD_TYPER: usize = GICD_BASE + 0x004;
const GICD_IIDR: usize = GICD_BASE + 0x008;
const GICD_IGROUPR: usize = GICD_BASE + 0x080;
const GICD_ISENABLER: usize = GICD_BASE + 0x100;
const GICD_ICENABLER: usize = GICD_BASE + 0x180;
const GICD_ISPENDR: usize = GICD_BASE + 0x200;
const GICD_ICPENDR: usize = GICD_BASE + 0x280;
const GICD_ISACTIVER: usize = GICD_BASE + 0x300;
const GICD_ICACTIVER: usize = GICD_BASE + 0x380;
const GICD_IPRIORITYR: usize = GICD_BASE + 0x400;
const GICD_ITARGETSR: usize = GICD_BASE + 0x800;
const GICD_ICFGR: usize = GICD_BASE + 0xC00;
const GICD_PPISR: usize = GICD_BASE + 0xD00;
const GICD_SGIR: usize = GICD_BASE + 0xF00;
const GICD_SGIR_CLRPEND: usize = GICD_BASE + 0xF10;
const GICD_SGIR_SETPEND: usize = GICD_BASE + 0xF20;
/* GICC Registers */
const GICC_CTLR: usize = GICC_BASE + 0x0000;
const GICC_PMR: usize = GICC_BASE + 0x0004;
const GICC_BPR: usize = GICC_BASE + 0x0008;
const GICC_IAR: usize = GICC_BASE + 0x000C;
const GICC_EOIR: usize = GICC_BASE + 0x0010;
const GICC_APR: usize = GICC_BASE + 0x00D0;
const GICC_IIDR: usize = GICC_BASE + 0x00FC;
const GICC_DIR: usize = GICC_BASE + 0x1000;

const GICD_CTL_ENABLE: usize = 0x1;
const GICD_CTL_DISABLE: usize = 0x0;
const GICD_INT_ACTLOW_LVLTRIG: usize = 0x0;
const GICD_INT_EN_CLR_X32: usize = 0xffffffff;
const GICD_INT_EN_SET_SGI: usize = 0x0000ffff;
const GICD_INT_EN_CLR_PPI: usize = 0xffff0000;
const GICD_INT_DEF_PRI: usize = 0xa0;
const GICD_INT_DEF_PRI_X4: usize = (GICD_INT_DEF_PRI << 24)
    | (GICD_INT_DEF_PRI << 16)
    | (GICD_INT_DEF_PRI << 8)
    | GICD_INT_DEF_PRI;

/* Register bits */
const GICD_TYPE_LINES: usize = 0x01F;
const GICD_TYPE_CPUS_SHIFT: usize = 5;
const GICD_TYPE_CPUS: usize = 0x0E0;
const GICD_TYPE_SEC: usize = 0x400;

const GICC_ENABLE: usize = 0x1;
const GICC_INT_PRI_THRESHOLD: usize = 0xf0;

const GICC_CTRL_EOImodeNS_SHIFT: usize = 9;
const GICC_CTRL_EOImodeNS: usize = 1 << GICC_CTRL_EOImodeNS_SHIFT;

const GICC_IAR_INT_ID_MASK: usize = 0x3ff;
const GICC_INT_SPURIOUS: usize = 1023;
const GICC_DIS_BYPASS_MASK: usize = 0x1e0;

const GIC_PRI_IRQ: usize = 0xA0;
const GIC_PRI_IPI: usize = 0x90;

/* GICD_SGIR defination */
const GICD_SGIR_SGIINTID_SHIFT: usize = 0;
const GICD_SGIR_CPULIST_SHIFT: usize = 16;
const GICD_SGIR_LISTFILTER_SHIFT: usize = 24;
/*
const GICD_SGIR_VAL(listfilter, cpulist, sgi)         \
    (((listfilter) << GICD_SGIR_LISTFILTER_SHIFT) | \
     ((cpulist) << GICD_SGIR_CPULIST_SHIFT) |       \
     ((sgi) << GICD_SGIR_SGIINTID_SHIFT))
*/
const GIC_INTID_EL1_PHYS_TIMER: usize = 30;
const GIC_INTID_EL3_PHYS_TIMER: usize = 29;
const GIC_INTID_VIRT_TIMER: usize = 27;
const GIC_INTID_EL2_PHYS_TIMER: usize = 26;

static mut nr_lines: u32 = 0;

#[inline]
fn put32(addr: usize, val: u32) {
    unsafe {
        (addr as *mut u32).write(val);
    }
}
#[inline]
fn get32(addr: usize) -> u32 {
    unsafe { (addr as *mut u32).read() }
}

fn gicv2_get_cpumask() -> u32 {
    let mut mask = 0;
    for i in (0..32).step_by(4) {
        mask = get32(GICD_ITARGETSR + i);
        mask |= mask >> 16;
        mask |= mask >> 8;
        if mask != 0 {
            break;
        }
    }
    return mask;
}

fn gicv2_dist_init() {
    /* Disable the distributor */
    put32(GICD_CTLR, GICD_CTL_DISABLE as u32);
    println!("disbale distributor");

    let _type = get32(GICD_TYPER);
    let lines: usize;
    unsafe {
        nr_lines = get32(GICD_TYPER) & GICD_TYPE_LINES as u32;
        nr_lines = (nr_lines + 1) * 32;
        lines = nr_lines as usize;
    }

    /* Set all global interrupts to this CPU only */
    let mut cpumask = gicv2_get_cpumask();
    cpumask |= cpumask << 8;
    cpumask |= cpumask << 16;
    for i in (32..lines as usize).step_by(4) {
        put32(GICD_ITARGETSR + i * 4 / 4, cpumask);
    }

    /* Set all global interrupts to be level triggered, active low */
    for i in (32..lines as usize).step_by(16) {
        put32(GICD_ICFGR + i / 4, GICD_INT_ACTLOW_LVLTRIG as u32);
    }

    /* Set priority on all global interrupts */
    for i in (32..lines as usize).step_by(4) {
        put32(GICD_IPRIORITYR + i, GICD_INT_DEF_PRI_X4 as u32);
    }

    /*
     * Deactivate and disable all SPIs. Leave the PPI and SGIs
     * alone as they are in the redistributor registers on GICv3.
     */
    for i in (32..lines as usize).step_by(32) {
        put32(GICD_ICACTIVER + i / 8, GICD_INT_EN_CLR_X32 as u32);
        put32(GICD_ICENABLER + i / 8, GICD_INT_EN_CLR_X32 as u32);
    }

    /* Turn on the distributor */
    put32(GICD_CTLR, GICD_CTL_ENABLE as u32);
}

fn gicv2_cpu_init() {
    /*
     * Deal with the banked PPI and SGI interrupts - disable all
     * private interrupts. Make sure everything is deactivated.
     */
    for i in (0..32).step_by(32) {
        put32(GICD_ICACTIVER + i / 8, GICD_INT_EN_CLR_X32 as u32);
        put32(GICD_ICENABLER + i / 8, GICD_INT_EN_CLR_X32 as u32);
    }

    /* Set priority on PPI and SGI interrupts */
    for i in (0..32).step_by(4) {
        put32(GICD_IPRIORITYR + i * 4 / 4, GICD_INT_DEF_PRI_X4 as u32);
    }

    /* Ensure all SGI interrupts are now enabled */
    put32(GICD_ISENABLER, GICD_INT_EN_SET_SGI as u32);

    /* Don't mask by priority */
    put32(GICC_PMR, GICC_INT_PRI_THRESHOLD as u32);

    /* Finest granularity of priority */
    put32(GICC_BPR, 0);
    for i in 0..4 {
        put32(GICC_APR + i * 4, 0);
    }

    /* Turn on delivery */
    let mut bypass = get32(GICC_CTLR);
    bypass &= GICC_DIS_BYPASS_MASK as u32;
    put32(
        GICC_CTLR,
        bypass | GICC_CTRL_EOImodeNS as u32 | GICC_ENABLE as u32,
    );
}

pub fn gicv2_init() {
    let cpuid = 0;
    if cpuid == 0 {
        gicv2_dist_init();
        println!("init gicv2 distribution");
    }
    /* init the cpu interface (GICC) */
    gicv2_cpu_init();
    println!("init gicv2 cpu interface");
    /* enable PPI irq */
    for i in (0..32).step_by(32) {
        put32(GICD_ISENABLER + i / 8, GICD_INT_EN_CLR_PPI as u32);
    }
    /* enable PL011 irq */
    put32(GICD_ISENABLER + 4, 0xF as u32);
}

pub fn gicvc2_handler() {
    let irqstat = get32(GICC_IAR);
    let irqnr = irqstat & 0xff_ffff;
    let mut sp = KernelAddr::new(0);
    match irqnr {
        30 => {
            super::timer::timer_irq();
        }
        33 => {
            super::pl011::pl011_irq_handler();
        }
        _ => {
            panic!("unsupported irq:{}", irqnr);
        }
    }
    put32(GICC_EOIR, irqstat);
    put32(GICC_DIR, irqstat);
    if irqnr == 30 {
        _yield();
    }
}

pub fn gicv2_disable() {
    let old = get32(GICC_CTLR);
    put32(GICC_CTLR, old & (!0x1));
}

pub fn gicv2_enable() {
    let old = get32(GICC_CTLR);
    put32(GICC_CTLR, old | 1);
}
