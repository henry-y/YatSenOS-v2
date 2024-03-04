use super::ProcessId;
use super::*;
use crate::memory::*;
use alloc::string::String;
use alloc::sync::Arc;
use alloc::sync::Weak;
use alloc::vec::Vec;
use spin::*;
use x86_64::structures::paging::mapper::MapToError;
use x86_64::structures::paging::page::PageRange;
use x86_64::structures::paging::*;
use x86_64::VirtAddr;
use elf;


#[derive(Clone)]
pub struct Process {
    pid: ProcessId,
    inner: Arc<RwLock<ProcessInner>>,
}

pub struct ProcessInner {
    name: String,
    parent: Option<Weak<Process>>,
    children: Vec<Arc<Process>>,
    ticks_passed: usize,
    status: ProgramStatus,
    exit_code: Option<isize>,
    context: ProcessContext,
    page_table: Option<PageTableContext>,
    proc_data: Option<ProcessData>,
}

impl Process {
    #[inline]
    pub fn pid(&self) -> ProcessId {
        self.pid
    }

    #[inline]
    pub fn write(&self) -> RwLockWriteGuard<ProcessInner> {
        self.inner.write()
    }

    #[inline]
    pub fn read(&self) -> RwLockReadGuard<ProcessInner> {
        self.inner.read()
    }

    pub fn new(
        name: String,
        parent: Option<Weak<Process>>,
        page_table: PageTableContext,
        proc_data: Option<ProcessData>,
    ) -> Arc<Self> {
        let name = name.to_ascii_lowercase();

        // create context
        let pid = ProcessId::new();

        let inner = ProcessInner {
            name,
            parent,
            status: ProgramStatus::Ready,
            context: ProcessContext::default(),
            ticks_passed: 0,
            exit_code: None,
            children: Vec::new(),
            page_table: Some(page_table),
            proc_data: Some(proc_data.unwrap_or_default()),
        };

        trace!("New process {}#{} created.", &inner.name, pid);

        // create process struct
        Arc::new(Self {
            pid,
            inner: Arc::new(RwLock::new(inner)),
        })
    }

    pub fn kill(&self, ret: isize) {
        let mut inner = self.inner.write();

        debug!(
            "Killing process {}#{} with ret code: {}",
            inner.name(),
            self.pid,
            ret
        );

        inner.kill(ret);
    }

    pub fn alloc_init_stack(&self) -> VirtAddr {
        // FIXME: alloc init stack base on self pid
        // 4GiB = 0x1000_0000
        // 2MiB = 0x200_000
        let pid : u16 = self.pid().into();
        let vaddr = STACK_INIT_BOT - (pid - 1) as u64 * 0x1_0000_0000;
        trace!("Alloc init stack: pid:{}: {:#?}", pid, vaddr);
        trace!("the all stack range is: [{:#x},{:#x})", 
            STACK_MAX - (pid) as u64 * 0x1_0000_0000, 
            STACK_MAX - (pid-1) as u64 * 0x1_0000_0000);
        let frame_allocator = 
            &mut *get_frame_alloc_for_sure();
        
        trace!("get_frame_allocator succ...");

        let page_range = 
            elf::map_range(vaddr, STACK_DEF_PAGE, 
            &mut self.read().page_table.as_ref().unwrap().mapper(), 
            frame_allocator, 
            false);

        trace!("mapper succ...");
        
        let rt_addr = page_range.unwrap();

        self.write().set_stack(rt_addr.start.start_address(), STACK_DEF_PAGE);
        self.write().set_max_stack(VirtAddr::new(STACK_MAX - (pid) as u64 * 0x1_0000_0000), 
                                    VirtAddr::new(STACK_MAX - (pid-1) as u64 * 0x1_0000_0000));
        // 对页面进行加，而不是对addr进行加，重载了加法

        // trace!("Alloc init stack's begin is: {:#?}", rt_addr.end.start_address());
        // trace!("stack range is: [{:#x},{:#x})", rt_addr.start.start_address(), rt_addr.start.start_address()+STACK_DEF_SIZE);
        // trace!("vaddr is {:#x}", vaddr);
        rt_addr.end.start_address()
    }
}

impl ProcessInner {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn tick(&mut self) {
        self.ticks_passed += 1;
    }

    pub fn status(&self) -> ProgramStatus {
        self.status
    }

    pub fn pause(&mut self) {
        self.status = ProgramStatus::Ready;
    }

    pub fn resume(&mut self) {
        self.status = ProgramStatus::Running;
    }

    pub fn exit_code(&self) -> Option<isize> {
        self.exit_code
    }

    pub fn clone_page_table(&self) -> PageTableContext {
        self.page_table.as_ref().unwrap().clone()
    }

    pub fn is_ready(&self) -> bool {
        self.status == ProgramStatus::Ready
    }

    pub fn set_init_stack(&mut self, stack_top: VirtAddr, entry: VirtAddr) {
        self.context.init_stack_frame(entry, stack_top)
    }

    pub fn set_user_init_stack(&mut self, stack_top: VirtAddr, entry: VirtAddr) {
        self.context.init_user_stack_frame(entry, stack_top)
    }

    pub fn update_stack_frame(&mut self, visit_addr: VirtAddr, user_access: bool) {
        let now_begin = self.stack_segment.unwrap().start.start_address();
        let visit_page = Page::<Size4KiB>::containing_address(visit_addr);
        let top_page = Page::<Size4KiB>::containing_address(now_begin);
        let page_count = top_page - visit_page;
        self.expand(visit_page.start_address(), page_count, user_access);

        self.stack_segment = Some(Page::range(visit_page, self.stack_segment.unwrap().end));
    }

    fn expand(&self, start: VirtAddr, count: u64, user_access: bool) {
        let frame_allocator = 
            &mut *get_frame_alloc_for_sure();
        
        elf::map_range(start.as_u64(), 
            count, 
            &mut self.page_table.as_ref().unwrap().mapper(), 
            frame_allocator,
            user_access).expect("map range err");
    }

    /// Save the process's context
    /// mark the process as ready
    pub(super) fn save(&mut self, context: &ProcessContext) {
        // FIXME: save the process's context
        self.context.save(context);
        if self.status != ProgramStatus::Dead {
            // 注意这里的逻辑，最后啥都做完的时候卡了两小时就是因为这里save的时候没判断是否dead
            self.pause();
        }
    }

    /// Restore the process's context
    /// mark the process as running
    pub(super) fn restore(&mut self, context: &mut ProcessContext) {
        // FIXME: restore the process's context
        self.context.restore(context);
        
        // trace!("Restoring process {}, is dead?: {}, is_ready? {}", self.name(), 
        //    self.status == ProgramStatus::Dead, self.is_ready());
            
        // FIXME: restore the process's page table
        self.page_table.as_ref().expect("get_page_table_err").load();
        
        if self.status != ProgramStatus::Dead {
            self.resume();
        }
    }

    pub fn parent(&self) -> Option<Arc<Process>> {
        self.parent.as_ref().and_then(|p| p.upgrade())
    }

    pub fn kill(&mut self, ret: isize) {
        // FIXME: set exit code
        self.exit_code = Some(ret);
        // FIXME: set status to dead
        self.status = ProgramStatus::Dead;
        // FIXME: take and drop unused resources
        drop(self.page_table.take());
        drop(self.proc_data.take());
        trace!("Process {} is dead., status: {:?}", self.name(), self.status);
    }

    /// load elf to process pagetable
    pub fn load_elf(&mut self, elf: &ElfFile) -> Result<(), MapToError<Size4KiB>> {

        let frame_allocator = &mut *get_frame_alloc_for_sure();
        let mut mapper = self.page_table.as_ref().unwrap().mapper();
        
        let code_segments = elf::load_elf(
            elf,
            *PHYSICAL_OFFSET.get().unwrap(),
            &mut mapper,
            frame_allocator,
            true,
        ).unwrap();
        
        let stack_segment = elf::map_range(STACK_INIT_BOT,
            STACK_DEF_PAGE, &mut mapper, frame_allocator, true).unwrap();
    
        self.set_stack(stack_segment.start.start_address(), STACK_DEF_PAGE);
        self.set_max_stack(VirtAddr::new(STACK_INIT_TOP - 0x1_0000_0000), 
            VirtAddr::new(STACK_INIT_TOP+8));
        Ok(())
    }
    

}

impl core::ops::Deref for Process {
    type Target = Arc<RwLock<ProcessInner>>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl core::ops::Deref for ProcessInner {
    type Target = ProcessData;

    fn deref(&self) -> &Self::Target {
        self.proc_data
            .as_ref()
            .expect("Process data empty. The process may be killed.")
    }
}

impl core::ops::DerefMut for ProcessInner {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.proc_data
            .as_mut()
            .expect("Process data empty. The process may be killed.")
    }
}

impl core::fmt::Debug for Process {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        let mut f = f.debug_struct("Process");
        f.field("pid", &self.pid);

        let inner = self.inner.read();
        f.field("name", &inner.name);
        f.field("parent", &inner.parent().map(|p| p.pid));
        f.field("status", &inner.status);
        f.field("ticks_passed", &inner.ticks_passed);
        f.field(
            "children",
            &inner.children.iter().map(|c| c.pid.0).collect::<Vec<u16>>(),
        );
        f.field("page_table", &inner.page_table);
        f.field("status", &inner.status);
        f.field("context", &inner.context);
        f.field("stack", &inner.proc_data.as_ref().map(|d| d.stack_segment));
        f.finish()
    }
}

impl core::fmt::Display for Process {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        let inner = self.inner.read();
        write!(
            f,
            " #{:-3} | #{:-3} | {:12} | {:7} | {:?}",
            self.pid.0,
            inner.parent().map(|p| p.pid.0).unwrap_or(0),
            inner.name,
            inner.ticks_passed,
            inner.status
        )?;
        Ok(())
    }
}
