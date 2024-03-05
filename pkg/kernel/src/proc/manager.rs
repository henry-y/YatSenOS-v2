use super::*;
use crate::memory::{
    self,
    allocator::{ALLOCATOR, HEAP_SIZE},
    get_frame_alloc_for_sure, PAGE_SIZE,
};
use alloc::{collections::BTreeMap, sync::Weak};
use alloc::{collections::VecDeque, format, sync::Arc};
use bitflags::Flags;
use spin::{Mutex, RwLock};
use x86_64::VirtAddr;

pub static PROCESS_MANAGER: spin::Once<ProcessManager> = spin::Once::new();

pub fn init(init: Arc<Process>, app_list: boot::AppListRef) {

    // FIXME: set init process as Running
    // get process inner
    let mut inner = init.write();
    inner.restore(&mut ProcessContext::default());
    inner.resume();

    drop(inner);
    // FIXME: set processor's current pid to init's pid
    crate::proc::processor::set_pid(init.pid());
    PROCESS_MANAGER.call_once(|| ProcessManager::new(init, app_list));
}

pub fn get_process_manager() -> &'static ProcessManager {
    PROCESS_MANAGER
        .get()
        .expect("Process Manager has not been initialized")
}

pub struct ProcessManager {
    processes: RwLock<BTreeMap<ProcessId, Arc<Process>>>,
    ready_queue: Mutex<VecDeque<ProcessId>>,
    app_list: Option<boot::AppListRef>,
} 

impl ProcessManager {
    pub fn new(init: Arc<Process>, app_list: boot::AppListRef) -> Self {
        let mut processes = BTreeMap::new();
        let ready_queue = VecDeque::new();
        let pid = init.pid();

        trace!("Init {:#?}", init);

        processes.insert(pid, init);
        Self {
            processes: RwLock::new(processes),
            ready_queue: Mutex::new(ready_queue),
            app_list: Some(app_list)
        }
    }

    pub fn app_list(&self) -> Option<boot::AppListRef> {
        self.app_list
    }

    pub fn spawn(
        &self,
        elf: &ElfFile,
        name: String,
        parent: Option<Weak<Process>>,
        proc_data: Option<ProcessData>,
    ) -> ProcessId {
        
        let kproc = self.get_proc(&KERNEL_PID).unwrap();
        let page_table = kproc.read().clone_page_table();
        let proc = Process::new(name, parent, page_table, proc_data);
        let pid = proc.pid();

        let mut inner = proc.write();
        
        trace!("begin load elf...");

        trace!("the elf hd msg: {:#?}", elf.header);

        // FIXME: load elf to process pagetable
        inner.load_elf(elf).expect("load_elf error");
        
        trace!("load elf succ...");

        // FIXME: alloc new stack for process
        inner.set_user_init_stack( 
            VirtAddr::new(STACK_INIT_TOP),
            VirtAddr::new(elf.header.pt2.entry_point())
        );
        // FIXME: mark process as ready
        inner.pause();

        drop(inner);
    
        trace!("New {:#?}", &proc);

        // FIXME: something like kernel thread
        self.add_proc(pid, proc);
        self.push_ready(pid);

        pid
    }

    #[inline]
    pub fn push_ready(&self, pid: ProcessId) {
        self.ready_queue.lock().push_back(pid);
    }

    #[inline]
    fn add_proc(&self, pid: ProcessId, proc: Arc<Process>) {
        self.processes.write().insert(pid, proc);
    }

    #[inline]
    fn get_proc(&self, pid: &ProcessId) -> Option<Arc<Process>> {
        self.processes.read().get(pid).cloned()
    }

    pub fn get_exit_code(&self, pid: ProcessId) -> Option<isize> {
        if self.get_proc(&pid).unwrap().read().status() == ProgramStatus::Dead {
            self.get_proc(&pid).unwrap().read().exit_code()
        } else {
            None
        }
    }

    pub fn get_page_fault_generator(&self) -> ProcessId {
        self.current().pid()
    }

    pub fn current(&self) -> Arc<Process> {
        self.get_proc(&processor::get_pid())
            .expect("No current process")
    }

    pub fn save_current(&self, context: &ProcessContext) {
        // FIXME: update current process's tick count
        let cur = self.current();
        let mut inner = cur.write();
        inner.tick();
        
        // FIXME: update current process's context
        inner.save(context);
        // FIXME: push current process to ready queue if still alive
        
        if inner.status() != ProgramStatus::Dead {
            drop(inner);
            self.push_ready(cur.pid());
        } 

        // if cur.read().name() == "stack" {
        //     trace!("print stack process in save_curent: {:?}", cur);
        // }
        
    }

    pub fn switch_next(&self, context: &mut ProcessContext) -> ProcessId {

        // FIXME: fetch the next process from ready queue
        let mut cur_pid = processor::get_pid();

        while let Some(next_pid) = self.ready_queue.lock().pop_front() {
            let proc = self.get_proc(&next_pid);
            if proc.is_none() {
                continue;
            }
            let proc = proc.unwrap();

            if proc.read().status() != ProgramStatus::Ready {
                trace!("dead process: {} {:?}", next_pid, proc.read().status());
                continue;
            }

            let mut inner = proc.write();
            inner.restore(context);
            drop(inner);

            processor::set_pid(next_pid);
            cur_pid = next_pid;

            break;

        }

        cur_pid


    }

    // pub fn spawn_kernel_thread(
    //     &self,
    //     entry: VirtAddr,
    //     name: String,
    //     proc_data: Option<ProcessData>,
    // ) -> ProcessId {
    //     let kproc = self.get_proc(&KERNEL_PID).unwrap();
    //     let page_table = kproc.read().clone_page_table();
    //     let proc = Process::new(name, Some(Arc::downgrade(&kproc)), page_table, proc_data);

    //     // alloc stack for the new process base on pid
    //     let stack_top = proc.alloc_init_stack();

    //     // FIXME: set the stack frame
    //     let mut inner = proc.write();
    //     inner.pause();
    //     inner.set_init_stack(stack_top, entry);
    //     drop(inner);
    //     // FIXME: add to process map
    //     let pid = proc.pid();
        
    //     self.add_proc(pid, proc);
    //     // FIXME: push to ready queue
    //     self.push_ready(pid);
        
    //     pid
    // }

    pub fn kill_self(&self, ret: isize) {
        self.kill(processor::get_pid(), ret);
    }

    pub fn kill_current(&self, ret: isize) {
        self.kill(processor::get_pid(), ret);
    }

    pub fn handle_page_fault(&self, addr: VirtAddr, err_code: PageFaultErrorCode) -> bool {
        // FIXME: handle page fault

        debug!("Page fault: addr: {:#x}, err_code: {:?}", addr, err_code);

        if !self.current().read().is_on_stack(addr) || err_code.contains(PageFaultErrorCode::PROTECTION_VIOLATION) {
            
            if !self.current().read().is_on_stack(addr) {
                trace!("Page fault: not on stack");
            } else {
                trace!("Page fault: protection violation");
            }

            return false;

        } else {
            self.current().write().update_stack_frame(addr, 
                self.current().pid() != ProcessId(0));
        }
        true
    }

    // [0x3ff900000000,0x3ffa00000000) 
    // 0x3ff9ffff7f90

    pub fn kill(&self, pid: ProcessId, ret: isize) {
        let proc = self.get_proc(&pid);

        if proc.is_none() {
            warn!("Process #{} not found.", pid);
            return;
        }

        let proc = proc.unwrap();

        if proc.read().status() == ProgramStatus::Dead {
            warn!("Process #{} is already dead.", pid);
            return;
        }

        trace!("Kill {:#?}", &proc);

        proc.kill(ret);
    }

    pub fn print_process_list(&self) {
        let mut output = String::from("  PID | PPID | Process Name |  Ticks  | Status\n");

        for (_, p) in self.processes.read().iter() {
            if p.read().status() != ProgramStatus::Dead {
                output += format!("{}\n", p).as_str();
            }
        }

        // TODO: print memory usage of kernel heap

        output += format!("Queue  : {:?}\n", self.ready_queue.lock()).as_str();

        output += &processor::print_processors();

        print!("{}", output);
    }

    pub fn still_alive(&self, pid: ProcessId) -> bool {
        let proc = get_process_manager().get_proc(&pid);
        if proc.is_none() {
            return false;
        }
        proc.unwrap().read().status() != ProgramStatus::Dead
    }

}
