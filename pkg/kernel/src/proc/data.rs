use alloc::{collections::BTreeMap, string::String, sync::Arc};
use spin::RwLock;
use x86_64::{
    structures::paging::{page::{PageRange}, Page},
    VirtAddr,
};
use crate::utils::resource::{Resource, StdIO};


#[derive(Debug, Clone)]
pub struct ProcessData {
    // shared data
    pub(super) env: Arc<RwLock<BTreeMap<String, String>>>,

    // process specific data
    pub(super) stack_segment: Option<PageRange>,
    pub(super) max_stack_segment: Option<PageRange>,
    pub(super) file_handles: Arc<RwLock<BTreeMap<u8, Resource>>>
}

impl Default for ProcessData {
    fn default() -> Self {
        let mut file_handles = BTreeMap::new();

        // stdin, stdout, stderr
        file_handles.insert(0, Resource::Console(StdIO::Stdin));
        file_handles.insert(1, Resource::Console(StdIO::Stdout));
        file_handles.insert(2, Resource::Console(StdIO::Stderr));

        Self {
            env: Arc::new(RwLock::new(BTreeMap::new())),
            stack_segment: None,
            max_stack_segment: None,
            file_handles: Arc::new(RwLock::new(file_handles)),
        }

    }
}

impl ProcessData {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn env(&self, key: &str) -> Option<String> {
        self.env.read().get(key).cloned()
    }

    pub fn set_env(&mut self, key: &str, val: &str) {
        self.env.write().insert(key.into(), val.into());
    }

    pub fn set_stack(&mut self, start: VirtAddr, size: u64) {
        let start = Page::containing_address(start);
        // trace!("in set_stack: {:#x}, {:#x}", start.start_address(), start.end_address());
        self.stack_segment = Some(Page::range(start, start + size));
    }

    pub fn set_max_stack(&mut self, start: VirtAddr, end: VirtAddr) {
        let start = Page::containing_address(start);
        let end = Page::containing_address(end);
        // trace!("in set_max_stack: {:#x}, {:#x}", start.start_address(), start.end_address());
        self.max_stack_segment = Some(Page::range(start, end));
    }

    pub fn is_on_stack(&self, addr: VirtAddr) -> bool {
        // FIXME: check if the address is on the stack
        // check if the address is in the stack segment
        // info!("stack testing...");
        if let Some(stack) = &self.max_stack_segment {
            trace!("testing is_on_stack: {:#x}, {:#x}, addr is {:#x}\n, compare result is {} {}", stack.start.start_address(), 
                stack.end.start_address(), addr,
                (stack.start.start_address() <= addr),
                 (addr < stack.end.start_address())
            );
            stack.start.start_address() <= addr && addr < stack.end.start_address()
        } else {
            false
        }
    }

    pub fn handle(&self, fd: u8) -> Option<Resource> {
        self.file_handles.read().get(&fd).cloned()
    }

    pub fn get_stack_page(&self) -> u64 {
        let tmp = self.stack_segment.as_ref().unwrap();
        return tmp.end - tmp.start;
    }


}
