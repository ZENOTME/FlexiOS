use alloc::vec::Vec;
use lazy_static::*;

use crate::{
    addr_space::VmSpace,
    addr_type::{Addr, UserAddr},
    arch::paging::PageTableFlags,
    frame::{DataFrame, FrameObj},
    frame_allocator::{FrameAllocator, CURRENT_FRAME_ALLOCATOR},
};

pub fn get_num_app() -> usize {
    extern "C" {
        fn _num_app();
    }
    unsafe { (_num_app as usize as *const usize).read_volatile() }
}

pub fn get_app_data(app_id: usize) -> &'static [u8] {
    extern "C" {
        fn _num_app();
    }
    let num_app_ptr = _num_app as usize as *const usize;
    let num_app = get_num_app();
    let app_start = unsafe { core::slice::from_raw_parts(num_app_ptr.add(1), num_app + 1) };
    assert!(app_id < num_app);
    unsafe {
        core::slice::from_raw_parts(
            app_start[app_id] as *const u8,
            app_start[app_id + 1] - app_start[app_id],
        )
    }
}

lazy_static! {
    static ref APP_NAMES: Vec<&'static str> = {
        let num_app = get_num_app();
        extern "C" {
            fn _app_names();
        }
        let mut start = _app_names as usize as *const u8;
        let mut v = Vec::new();
        unsafe {
            for _ in 0..num_app {
                let mut end = start;
                while end.read_volatile() != '\0' as u8 {
                    end = end.add(1);
                }
                let slice = core::slice::from_raw_parts(start, end as usize - start as usize);
                let str = core::str::from_utf8(slice).unwrap();
                v.push(str);
                start = end.add(1);
            }
        }
        v
    };
}

#[allow(unused)]
pub fn get_app_data_by_name(name: &str) -> Option<&'static [u8]> {
    let num_app = get_num_app();
    (0..num_app)
        .find(|&i| APP_NAMES[i] == name)
        .map(|i| get_app_data(i))
}

pub fn list_apps() {
    println!("/**** APPS ****");
    for app in APP_NAMES.iter() {
        println!("{}", app);
    }
    println!("**************/");
}

pub fn elf_mapper(elf_data: &[u8], space: &mut VmSpace) -> UserAddr {
    let elf = xmas_elf::ElfFile::new(elf_data).unwrap();
    let elf_header = elf.header;
    let magic = elf_header.pt1.magic;
    assert_eq!(magic, [0x7f, 0x45, 0x4c, 0x46], "invalid elf!");
    let ph_count = elf_header.pt2.ph_count();
    for i in 0..ph_count {
        //info!("Loader Hello Elf Segement :{}",i);
        let ph = elf.program_header(i).unwrap();
        //info!("===========");
        //info!("{:?}",ph);
        if ph.get_type().unwrap() == xmas_elf::program::Type::Load {
            let start_va: UserAddr = UserAddr::new(ph.virtual_addr());
            let end_va: UserAddr = UserAddr::new(ph.virtual_addr() + ph.mem_size());
            let mut flag = PageTableFlags::ATTR_INDEX.val(0)
                + PageTableFlags::SH::INNERSHARE
                + PageTableFlags::AF::SET;
            let ph_flags = ph.flags();
            //info!("{}",ph_flags);
            if ph_flags.is_read() && ph_flags.is_write() {
                //info!("RW!!!");
                flag = flag + PageTableFlags::AP::EL0_RW_ELX_RW;
            } else if ph_flags.is_read() {
                //info!("OR!!!");
                flag = flag + PageTableFlags::AP::EL0_OR_ELX_OR;
            }
            if !ph_flags.is_execute() {
                flag = flag + PageTableFlags::UXN::SET + PageTableFlags::PXN::SET;
            } else {
                flag = flag + PageTableFlags::UXN::CLEAR + PageTableFlags::PXN::SET;
            }

            let len = end_va.addr() - start_va.addr();
            let mut t = CURRENT_FRAME_ALLOCATOR
                .exclusive_access()
                .allocate_frames(start_va, len)
                .unwrap();
            let mut frames = Vec::new();
            //load data
            let data = &elf.input[ph.offset() as usize..(ph.offset() + ph.file_size()) as usize];
            copy_from_data(&mut t, data);
            //
            for _frame in t.into_iter() {
                frames.push(FrameObj::Data(_frame));
            }
            space.map_range(start_va.addr(), len, frames, Some(flag));
        }
    }
    UserAddr::new(elf.header.pt2.entry_point())
}

fn copy_from_data(frames: &mut Vec<DataFrame>, data: &[u8]) {
    let mut len = data.len();
    let mut pos = 0;
    for _f in frames.iter_mut() {
        let l = if len < (_f.frame_size() as usize) {
            len
        } else {
            _f.frame_size() as usize
        };
        let src = &data[pos..pos + l];
        let dst = &mut _f.as_slice_mut::<u8>(0, src.len() as u64).unwrap();
        dst.clone_from_slice(src);
        pos += l;
        len -= l;
        if len == 0 {
            break;
        }
    }
}
