use crate::evaluate::{ForthResult, Error, ForthState};
use super::memory;


const SMALLBIN_SIZE: usize = 0x200;
const SMALLBIN_STEP: usize = 0x10;
const LARGEBIN_SIZE: usize = 0x1000;
const LARGEBIN_STEP: usize = 0x80;

const RANGE_SIZE: usize = memory::PAGE_SIZE * PAGES_PER_RANGE;
const PAGES_PER_RANGE: usize = 16;

struct PageRange {
    // address of the first page.  must start at some multiple of 0x10000
    start: memory::Address,
    // a page / the last page may not be fully used.  marks where the last allocation on the page ends
    empty_tail: usize,
    empty_tail_start: memory::Address,
    // the size of all chunks within this page range
    chunk_size: usize,
    // a list of all available chunks that are before the empty tail area
    available: Vec<memory::Address>,
}

impl PageRange {
    fn new(start: memory::Address, chunk_size: usize) -> Self {
        Self {
            start, chunk_size,
            empty_tail: RANGE_SIZE,
            empty_tail_start: start,
            available: Vec::new()
        }
    }

    fn is_full(&self) -> bool {
        self.empty_tail == 0 && self.available.len() == 0
    }

    fn allocate_next(&mut self) -> Result<memory::Address, Error> {
        if self.available.len() > 0 {
            Ok(self.available.pop().unwrap())
        } else if self.empty_tail > self.chunk_size {
            let address = self.empty_tail_start;

            self.empty_tail -= self.chunk_size;
            self.empty_tail_start.add(self.chunk_size);

            Ok(address)
        } else {
            Err(Error::InsufficientMemory)
        }
    }

    fn free(&mut self, address: memory::Address) {
        if address.plus(self.chunk_size).equals(self.empty_tail_start) {
            self.empty_tail_start.subtract(self.chunk_size);
            self.empty_tail += self.chunk_size;
        } else {
            self.available.push(address);
        }
    }
}

struct Bin {
    sections: Vec<Vec<PageRange>>,
    start_size: usize,
    end_size: usize,
    step: usize
}

impl Bin {
    fn new(start_size: usize, end_size: usize, step: usize) -> Self {
        let sections = Vec::new();
        for size in (start_size..end_size).step_by(step) {
            sections.push(Vec::new());
        }

        Self { sections, start_size, end_size, step }
    }

    fn get_page_ranges(&mut self, size: usize) -> Result<&mut Vec<PageRange>, Error> {
        if size < self.start_size || size >= self.end_size {
            return Err(Error::InvalidSize)
        }

        let index = (size - self.start_size + self.step - 1) / self.step;
        Ok(&mut self.sections[index]) 
    }
}

struct Heap {
    next_base: memory::Address,
    smallbin: Bin,
    largebin: Bin
}

impl Heap {
    pub fn new(base: usize) -> Self {
        Self {
            next_base: memory::Address::from_raw(base), 
            smallbin: Bin::new(0, SMALLBIN_SIZE, SMALLBIN_STEP),
            largebin: Bin::new(SMALLBIN_SIZE, LARGEBIN_SIZE, LARGEBIN_STEP)
        }
    }
}

struct LiveHeap<'a, 'b, 'c, 'd> {
    heap: &'a mut Heap,
    state: &'a mut ForthState<'b, 'c, 'd>,
}

impl<'a> LiveHeap<'a, '_, '_, '_> {
    pub fn allocate(&mut self, size: usize) -> Result<memory::Address, Error> {
        /*
        O(1)
        either fast or smallbin
        lookup by size
        get the first page range that has something available
            take from available list first, then empty tail
        move full page range(s) to the full list or swap first non full to front
        if no available page range is found, allocate a new one
        */
        let bin = if size < SMALLBIN_SIZE {
            &mut self.heap.smallbin
        } else if size < LARGEBIN_SIZE {
            &mut self.heap.largebin
        } else {
            panic!()
        };

        let table = bin.get_page_ranges(size)?;
        if table.len() == 0 {
            let next_base = self.heap.next_base;
            self.heap.next_base.add(RANGE_SIZE);

            let start = self.state.create_anonymous_mapping_at(next_base, RANGE_SIZE)?;
            table.push(PageRange::new(start, size));
        }
        let first_nonfull_index = None;
        for i in 0..table.len() {
            if !table[i].is_full() {
                first_nonfull_index = Some(i);
                break
            }
        }
    }

    pub fn free(address: memory::Address) -> ForthResult {
        /*
        O(1)
        pages are chunked together in groups of 16
        so 0xXXXXXXXX0000
        at that base address is the size of its chunks

        look for the page chunk its on within the sizes ( should be O(1) cause currently in use page is cached )
        use the same page chunk until its full
        */
        Ok(())
    }

    pub fn resize(address: memory::Address, size: usize) -> ForthResult {
        Ok(())
    }
}
