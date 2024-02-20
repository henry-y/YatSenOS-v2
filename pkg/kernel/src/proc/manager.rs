use super::*;
use crate::memory::{
    self,
    allocator::{ALLOCATOR, HEAP_SIZE},
    get_frame_alloc_for_sure, PAGE_SIZE,
};
use alloc::collections::BTreeMap;
use alloc::{collections::VecDeque, format, sync::Arc};
use spin::{Mutex, RwLock};
use x86_64::VirtAddr;

pub static PROCESS_MANAGER: spin::Once<ProcessManager> = spin::Once::new();

pub fn init(init: Arc<Process>) {

    // FIXME: set init process as Running
    // get process inner
    let mut inner = init.write();
    inner.restore(&mut ProcessContext::default());
    inner.resume();

    drop(inner);
    // FIXME: set processor's current pid to init's pid
    crate::proc::processor::set_pid(init.pid());
    PROCESS_MANAGER.call_once(|| ProcessManager::new(init));
}

pub fn get_process_manager() -> &'static ProcessManager {
    PROCESS_MANAGER
        .get()
        .expect("Process Manager has not been initialized")
}

pub struct ProcessManager {
    processes: RwLock<BTreeMap<ProcessId, Arc<Process>>>,
    ready_queue: Mutex<VecDeque<ProcessId>>,
} 

impl ProcessManager {
    pub fn new(init: Arc<Process>) -> Self {
        let mut processes = BTreeMap::new();
        let ready_queue = VecDeque::new();
        let pid = init.pid();

        trace!("Init {:#?}", init);

        processes.insert(pid, init);
        Self {
            processes: RwLock::new(processes),
            ready_queue: Mutex::new(ready_queue),
        }
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
        
    }

    pub fn switch_next(&self, context: &mut ProcessContext) -> ProcessId {

        // FIXME: fetch the next process from ready queue
        let mut next_pid = self.ready_queue.lock().pop_front().unwrap();
        // FIXME: check if the next process is ready,
        //        continue to fetch if not ready
        // get inner from btree
        let mut inner_status = 
            self.get_proc(&next_pid)
            .expect("Get Process In BTreeMap Err").read().status();
        while inner_status != ProgramStatus::Ready {
            self.push_ready(next_pid); 
            // 把这个重新扔进调度队列里面
            next_pid = self.ready_queue.lock().pop_front().unwrap();
            inner_status = 
                self.get_proc(&next_pid)
                .expect("Get Process In BTreeMap Err")
                .read().status();
        }
        // FIXME: restore next process's context
        self.get_proc(&next_pid)
            .expect("Get Process In BTreeMap Err").
            write().restore(context);

        // FIXME: update processor's current pid
        processor::set_pid(next_pid);
        next_pid
    }

    pub fn spawn_kernel_thread(
        &self,
        entry: VirtAddr,
        name: String,
        proc_data: Option<ProcessData>,
    ) -> ProcessId {
        let kproc = self.get_proc(&KERNEL_PID).unwrap();
        let page_table = kproc.read().clone_page_table();
        let proc = Process::new(name, Some(Arc::downgrade(&kproc)), page_table, proc_data);

        // alloc stack for the new process base on pid
        let stack_top = proc.alloc_init_stack();

        // FIXME: set the stack frame
        let mut inner = proc.write();
        inner.pause();
        inner.set_init_stack(stack_top, entry);
        drop(inner);
        // FIXME: add to process map
        let pid = proc.pid();
        
        self.add_proc(pid, proc);
        // FIXME: push to ready queue
        self.push_ready(pid);
        
        pid
    }

    pub fn kill_current(&self, ret: isize) {
        self.kill(processor::get_pid(), ret);
    }

    pub fn handle_page_fault(&self, addr: VirtAddr, err_code: PageFaultErrorCode) -> bool {
        // FIXME: handle page fault

        false
    }

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
}
