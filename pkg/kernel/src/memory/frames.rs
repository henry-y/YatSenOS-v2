use alloc::vec::Vec;
use boot::{MemoryMap, MemoryType};
use x86_64::structures::paging::{FrameAllocator, FrameDeallocator, PhysFrame, Size4KiB};
use x86_64::PhysAddr;

once_mutex!(pub FRAME_ALLOCATOR: BootInfoFrameAllocator);

guard_access_fn! {
    pub get_frame_alloc(FRAME_ALLOCATOR: BootInfoFrameAllocator)
}

type BootInfoFrameIter = impl Iterator<Item = PhysFrame>;

/// A FrameAllocator that returns usable frames from the bootloader's memory map.
pub struct BootInfoFrameAllocator {
    size: usize,
    used: usize,
    frames: BootInfoFrameIter,
    recycled: Vec<u32>
}

impl BootInfoFrameAllocator {
    /// Create a FrameAllocator from the passed memory map.
    ///
    /// This function is unsafe because the caller must guarantee that the passed
    /// memory map is valid. The main requirement is that all frames that are marked
    /// as `USABLE` in it are really unused.
    pub unsafe fn init(memory_map: &MemoryMap, size: usize) -> Self {
        BootInfoFrameAllocator {
            size,
            frames: create_frame_iter(memory_map),
            used: 0,
            recycled: Vec::new(),
        }
    }

    pub fn frames_used(&self) -> usize {
        self.used
    }

    pub fn frames_total(&self) -> usize {
        self.size
    }

    pub fn recycle_size(&self) -> usize {
        self.recycled.len()
    }
}

unsafe impl FrameAllocator<Size4KiB> for BootInfoFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame> {
        if self.recycle_size() == 0 {
            self.used += 1;
            self.frames.next()
        } else {
            let key = self.recycled.pop();
            let key = PhysFrame::from_start_address(PhysAddr::new(key.unwrap() as u64))
                .expect("recycle addr not aligned");
            Some(key)
        }
    }
}

impl FrameDeallocator<Size4KiB> for BootInfoFrameAllocator {
    unsafe fn deallocate_frame(&mut self, frame: PhysFrame) {
        // TODO: deallocate frame (not for lab 2)
        let key = frame.start_address().as_u64();
        assert!(key <= u32::MAX as u64);
        self.recycled.push(key as u32);
    }
}

unsafe fn create_frame_iter(memory_map: &MemoryMap) -> BootInfoFrameIter {
    memory_map
        .clone()
        .into_iter()
        // get usable regions from memory map
        .filter(|r| r.ty == MemoryType::CONVENTIONAL)
        // align to page boundary
        .flat_map(|r| (0..r.page_count).map(move |v| (v * 4096 + r.phys_start)))
        // create `PhysFrame` types from the start addresses
        .map(|addr| PhysFrame::containing_address(PhysAddr::new(addr)))
}
