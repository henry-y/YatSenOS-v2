use x86_64::{structures::paging::*, VirtAddr};
use xmas_elf::ElfFile;
use alloc::vec::Vec;

use crate::memory::*;

pub mod stack;

use self::stack::Stack;

use super::PageTableContext;

type MapperRef<'a> = &'a mut OffsetPageTable<'static>;
type FrameAllocatorRef<'a> = &'a mut BootInfoFrameAllocator;

pub struct ProcessVm {
    // page table is shared by parent and child
    pub(super) page_table: PageTableContext,

    // stack is pre-process allocated
    pub(super) stack: Stack,
    pub(super) code: Vec<u64>,

    pub(super) stack_usage: u64,
    pub(super) code_usage: u64
}

impl ProcessVm {
    pub fn new(page_table: PageTableContext) -> Self {
        Self {
            page_table,
            stack: Stack::empty(),
            code: Vec::new(),

            stack_usage: 0,
            code_usage: 0
        }
    }

    pub fn init_kernel_vm(mut self) -> Self {
        // TODO: record kernel code usage
        self.stack = Stack::kstack();
        self
    }

    pub fn load_elf(&mut self, elf: &ElfFile) {
        let mapper = &mut self.page_table.mapper();

        let alloc = &mut *get_frame_alloc_for_sure();

        self.load_elf_code(elf, mapper, alloc);
        self.stack.init(mapper, alloc);
    }

    fn load_elf_code(&mut self, elf: &ElfFile, mapper: MapperRef, alloc: FrameAllocatorRef) {
        elf::load_elf(elf, *PHYSICAL_OFFSET.get().unwrap(), mapper, alloc, true).ok();
    }

    pub fn handle_page_fault(&mut self, addr: VirtAddr) -> bool {
        let mapper = &mut self.page_table.mapper();
        let alloc = &mut *get_frame_alloc_for_sure();

        self.stack.handle_page_fault(addr, mapper, alloc)
    }

    pub fn fork(&self, stack_offset_count: u64) -> Self {
        // clone the page table context (see instructions)
        let owned_page_table = self.page_table.fork();

        let mapper = &mut owned_page_table.mapper();
        let alloc = &mut *get_frame_alloc_for_sure();

        Self {
            page_table: owned_page_table,
            stack: self.stack.fork(mapper, alloc, stack_offset_count),
            stack_usage: self.stack_usage,
            // do not share code info
            code: Vec::new(),
            code_usage: 0,
        }
    }

}

impl core::fmt::Debug for ProcessVm {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        f.debug_struct("ProcessVm")
            .field("stack", &self.stack)
            .field("page_table", &self.page_table)
            .finish()
    }
}
