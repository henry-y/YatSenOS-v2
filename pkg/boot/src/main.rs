#![no_std]
#![no_main]
#![feature(alloc_error_handler)]

#[macro_use]
extern crate log;
extern crate alloc;

use alloc::boxed::Box;
use alloc::vec;
use uefi::prelude::*;
use x86_64::registers::control::*;
use xmas_elf::ElfFile;
use ysos_boot::*;
use elf;

mod config;

const CONFIG_PATH: &str = "\\EFI\\BOOT\\boot.conf";

#[entry]
fn efi_main(image: uefi::Handle, mut system_table: SystemTable<Boot>) -> Status {
    uefi_services::init(&mut system_table).expect("Failed to initialize utilities");

    log::set_max_level(log::LevelFilter::Trace);
    info!("Running UEFI bootloader...");

    let bs = system_table.boot_services();

    // 1. Load config
    let config = { 
        /* FIXME: Load config file */         
        let mut file = open_file(bs, CONFIG_PATH);
        config::Config::parse(load_file(bs, &mut file))
    };

    info!("Config: {:#x?}", config);

    // 2. Load ELF files
    let elf = { 
        /* FIXME: Load kernel elf file */ 
        let mut file = open_file(bs, "\\KERNEL.ELF");
        let buf = load_file(bs, &mut file);
        ElfFile::new(buf).expect("load elf_file error")
    };

    unsafe {
        set_entry(elf.header.pt2.entry_point() as usize);
    }

    // 3. Load MemoryMap
    let max_mmap_size = system_table.boot_services().memory_map_size();
    let mmap_storage = Box::leak(
        vec![0; max_mmap_size.map_size + 10 * max_mmap_size.entry_size].into_boxed_slice(),
    );
    let mmap = system_table
        .boot_services()
        .memory_map(mmap_storage)
        .expect("Failed to get memory map");

    let max_phys_addr = mmap
        .entries()
        .map(|m| m.phys_start + m.page_count * 0x1000)
        .max()
        .unwrap()
        .max(0x1_0000_0000); // include IOAPIC MMIO area

    // 4. Map ELF segments, kernel stack and physical memory to virtual memory
    let mut page_table = current_page_table();

    // FIXME: root page table is readonly, disable write protect (Cr0)
    unsafe {
        Cr0::update(|f| f.remove(Cr0Flags::WRITE_PROTECT));
    }

    // FIXME: map physical memory to specific virtual address offset
    let mut frame_allocator = UEFIFrameAllocator(bs);
    elf::map_physical_memory(
        config.physical_memory_offset,
        max_phys_addr,
        &mut page_table,
        &mut frame_allocator
    );

    // FIXME: load and map the kernel elf file
    elf::load_elf(
        &elf, 
        config.physical_memory_offset, 
        &mut page_table, 
        &mut frame_allocator).expect("load elf error");

    // FIXME: map kernel stack
    // addressed_by_henry_y!!!: stack addr is an virtual address, so it need to map to page_table

    elf::map_physical_memory(
        config.kernel_stack_address,
        max_phys_addr,
        &mut page_table,
        &mut frame_allocator
    );

    // FIXME: recover write protect (Cr0)
    unsafe {
        Cr0::update(|f| f.insert(Cr0Flags::WRITE_PROTECT));
    }

    free_elf(bs, elf);

    // trace!("{:#?}", config);

    let apps = if config.load_apps {
            info!("Loading apps...");
            Some(load_apps(system_table.boot_services()))
        } else {
            info!("Skip loading apps");
            None
        };

    // 5. Exit boot and jump to ELF entry
    info!("Exiting boot services...");

    let (runtime, mmap) = system_table.exit_boot_services(MemoryType::LOADER_DATA);
    // NOTE: alloc & log are no longer available

    
    // construct BootInfo
    let bootinfo = BootInfo {
        memory_map: mmap.entries().copied().collect(),
        physical_memory_offset: config.physical_memory_offset,
        system_table: runtime,
        loaded_apps: apps,
    };

    // align stack to 8 bytes
    let stacktop = config.kernel_stack_address + config.kernel_stack_size * 0x1000 - 8;

    unsafe {
        jump_to_entry(&bootinfo, stacktop);
    }
}
