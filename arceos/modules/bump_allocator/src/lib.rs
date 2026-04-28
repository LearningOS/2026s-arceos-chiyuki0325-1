#![no_std]

use allocator::{BaseAllocator, ByteAllocator, PageAllocator};

/// Early memory allocator
/// Use it before formal bytes-allocator and pages-allocator can work!
/// This is a double-end memory range:
/// - Alloc bytes forward
/// - Alloc pages backward
///
/// [ bytes-used | avail-area | pages-used ]
/// |            | -->    <-- |            |
/// start       b_pos        p_pos       end
///
/// For bytes area, 'count' records number of allocations.
/// When it goes down to ZERO, free bytes-used area.
/// For pages area, it will never be freed!
///
pub struct EarlyAllocator<const PAGE_SIZE: usize> {
    start: usize,
    end: usize,
    b_pos: usize,
    p_pos: usize,
    count: usize,
}

impl<const PAGE_SIZE: usize> EarlyAllocator<PAGE_SIZE> {
    pub const fn new() -> Self {
        Self {
            start: 0,
            end: 0,
            b_pos: 0,
            p_pos: 0,
            count: 0,
        }
    }
}

impl<const PAGE_SIZE: usize> BaseAllocator for EarlyAllocator<PAGE_SIZE> {
    fn init(&mut self, start: usize, size: usize) {
        self.start = start;
        self.end = start + size;
        self.b_pos = start;
        self.p_pos = start + size;
    }

    fn add_memory(&mut self, start: usize, size: usize) -> allocator::AllocResult {
        unimplemented!()
    }
}

fn align_up_addr(addr: usize, align: usize) -> usize {
    (addr + align - 1) & !(align - 1)
}

fn align_down_addr(addr: usize, align: usize) -> usize {
    addr & !(align - 1)
}

impl<const PAGE_SIZE: usize> ByteAllocator for EarlyAllocator<PAGE_SIZE> {
    fn alloc(
        &mut self,
        layout: core::alloc::Layout,
    ) -> allocator::AllocResult<core::ptr::NonNull<u8>> {
        let alloc_start = align_up_addr(self.b_pos, layout.align());

        // 检查是否有足够的空间进行分配
        let alloc_end = alloc_start
            .checked_add(layout.size())
            .ok_or(allocator::AllocError::NoMemory)?;
        if alloc_end > self.p_pos {
            return Err(allocator::AllocError::NoMemory);
        }

        let slice = unsafe { core::slice::from_raw_parts_mut(alloc_start as *mut u8, layout.size()) };
        slice.fill(0);

        self.b_pos = alloc_end;
        self.count += 1;

        return Ok(core::ptr::NonNull::new(alloc_start as *mut u8).unwrap());
    }

    fn dealloc(&mut self, pos: core::ptr::NonNull<u8>, layout: core::alloc::Layout) {
        let alloc_start = pos.as_ptr() as usize;
        let alloc_end = align_up_addr(alloc_start, layout.align())
            .checked_add(layout.size())
            .expect("Overflow in dealloc");

        if alloc_start < self.start || alloc_end > self.b_pos {
            panic!("Out of bounds");
        }

        self.count -= 1;
        if self.count == 0 {
            let slice = unsafe {
                core::slice::from_raw_parts_mut(self.start as *mut u8, self.b_pos - self.start)
            };
            slice.fill(0);
            // reset b_pos
            self.b_pos = self.start;
        }
    }

    fn total_bytes(&self) -> usize {
        self.p_pos - self.start
    }

    fn used_bytes(&self) -> usize {
        self.b_pos - self.start
    }

    fn available_bytes(&self) -> usize {
        self.p_pos - self.b_pos
    }
}

impl<const PAGE_SIZE: usize> PageAllocator for EarlyAllocator<PAGE_SIZE> {
    const PAGE_SIZE: usize = PAGE_SIZE;

    fn alloc_pages(
        &mut self,
        num_pages: usize,
        align_pow2: usize, // must be a power of 2 and multiple of PAGE_SIZE
    ) -> allocator::AllocResult<usize> {
        if align_pow2 % Self::PAGE_SIZE != 0 || !align_pow2.is_power_of_two() {
            return Err(allocator::AllocError::InvalidParam);
        }
        let align_start = align_down_addr(self.p_pos - num_pages * Self::PAGE_SIZE, align_pow2);
        if align_start < self.b_pos {
            return Err(allocator::AllocError::NoMemory);
        }
        let size = num_pages * Self::PAGE_SIZE;
        let slice = unsafe { core::slice::from_raw_parts_mut(align_start as *mut u8, size) };
        slice.fill(0);
        self.p_pos = align_start;
        // 返回结果是 align_start
        Ok(align_start)
    }

    fn dealloc_pages(&mut self, _pos: usize, _num_pages: usize) {
        unimplemented!()
    }

    fn total_pages(&self) -> usize {
        self.end - self.b_pos / Self::PAGE_SIZE
    }

    fn used_pages(&self) -> usize {
        self.end - self.p_pos / Self::PAGE_SIZE
    }

    fn available_pages(&self) -> usize {
        self.total_pages() - self.used_pages()
    }
}
