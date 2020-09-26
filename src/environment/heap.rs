use crate::evaluate::{ForthResult, Error};
use crate::environment::{value::{Value}, generic_numbers::AsValue};
use super::memory::{self, MemorySegment};


const SMALLBIN_SIZE: usize = 0x200 / memory::CELL_SIZE;
const SMALLBIN_STEP: usize = 0x10 / memory::CELL_SIZE;
const LARGEBIN_SIZE: usize = 0x1000 / memory::CELL_SIZE;
const LARGEBIN_STEP: usize = 0x80 / memory::CELL_SIZE;

const PAGES_PER_RANGE: usize = 16;
const RANGE_SIZE: usize = memory::PAGE_SIZE * PAGES_PER_RANGE;
const MAX_VALUES_PER_RANGE: usize = memory::CELLS_PER_PAGE * PAGES_PER_RANGE;

struct PageRange {
    // address of the first page.  must start at some multiple of 0x10000
    base: memory::Address,
    // the array of actual memory
    memory: Vec<Value>,
    // the size of all chunks within this page range, in cells
    chunk_size: usize,
    // a list of all available chunks that are before the empty tail area
    available: Vec<memory::Address>,
}

impl PageRange {
    fn new(base: memory::Address, chunk_size: usize) -> Self {
        Self {
            base, chunk_size,
            memory: Vec::new(),
            available: Vec::new()
        }
    }

    fn is_full(&self) -> bool {
        self.memory.len() == MAX_VALUES_PER_RANGE && self.available.len() == 0
    }

    fn allocate_next(&mut self) -> Result<memory::Address, Error> {
        if self.available.len() > 0 {
            Ok(self.available.pop().unwrap())
        } else if MAX_VALUES_PER_RANGE - self.memory.len() >= self.chunk_size {
            let address = self.base.plus_cell(self.memory.len());
            self.memory.resize(self.memory.len() + self.chunk_size, 0.value());
            Ok(address)
        } else {
            Err(Error::InsufficientMemory)
        }
    }

    fn in_range(&self, address: memory::Address) -> bool {
        address.between(self.base, self.base.plus_cell(self.memory.len()))
    }

    fn free(&mut self, address: memory::Address) {
        if address.cell_offset_from(self.base) + self.chunk_size == self.memory.len() {
            self.memory.resize(self.memory.len() - self.chunk_size, 0.value());
        } else {
            self.available.push(address);
        }
    }

    fn write(&mut self, address: memory::Address, value: Value) {
        let offset = address.cell_offset_from(self.base);
        self.memory[offset] = value
    }

    fn read(&self, address: memory::Address) -> Value {
        let offset = address.cell_offset_from(self.base);
        self.memory[offset]
    }
}

// all of these sizes are in bytes
struct Bin {
    sections: Vec<Vec<PageRange>>,
    start_size: usize,
    end_size: usize,
    step: usize
}

impl Bin {
    fn new(start_size: usize, end_size: usize, step: usize) -> Self {
        let mut sections = Vec::new();
        for _ in (start_size..end_size).step_by(step) {
            sections.push(Vec::new());
        }

        Self { sections, start_size, end_size, step }
    }

    fn get_page_ranges_mut(&mut self, size: usize) -> Result<(usize, &mut Vec<PageRange>), Error> {
        if size < self.start_size || size >= self.end_size {
            return Err(Error::InvalidSize)
        }

        let local_size = ((size + self.step - 1) / self.step) * self.step;        

        let index = (local_size - self.start_size) / self.step;
        Ok((local_size, &mut self.sections[index]))
    }

    fn get_page_ranges(&self, size: usize) -> Result<(usize, &Vec<PageRange>), Error> {
        if size < self.start_size || size >= self.end_size {
            return Err(Error::InvalidSize)
        }

        let local_size = ((size + self.step - 1) / self.step) * self.step;        

        let index = (local_size - self.start_size) / self.step;
        Ok((local_size, &self.sections[index]))
    }
}

struct Bins {
    smallbin: Bin,
    largebin: Bin,
}

impl Bins {
    fn new() -> Self {
        Self {
            smallbin: Bin::new(0, SMALLBIN_SIZE, SMALLBIN_STEP),
            largebin: Bin::new(SMALLBIN_SIZE, LARGEBIN_SIZE, LARGEBIN_STEP),
        }
    }

    fn get_bin_mut(&mut self, size: usize) -> &mut Bin {
        if size < SMALLBIN_SIZE {
            &mut self.smallbin
        } else if size < LARGEBIN_SIZE {
            &mut self.largebin
        } else {
            panic!()
        }
    }

    fn get_bin(&self, size: usize) -> &Bin {
        if size < SMALLBIN_SIZE {
            &self.smallbin
        } else if size < LARGEBIN_SIZE {
            &self.largebin
        } else {
            panic!()
        }
    }
}

pub struct Heap {
    base: memory::Address,
    bins: Bins,
    size_lookup: Vec<usize>,
}

impl Heap {
    pub fn new(base: usize) -> Self {
        Self {
            base: memory::Address::from_raw(base), 
            bins: Bins::new(),
            size_lookup: Vec::new(),
        }
    }

    /**
     * Returns the address of the new allocation.
     * Size is specified in bytes.
     */
    pub fn allocate(&mut self, size: usize) -> Result<memory::Address, Error> {
        // convert the size from bytes to cells
        let size = (size + memory::CELL_SIZE - 1) / memory::CELL_SIZE; 

        // get the adjusted size, and corresponding table
        let (size, table) = self.bins.get_bin_mut(size).get_page_ranges_mut(size)?;

        // scan backwards for the first non-full entry
        let mut available_range = None;
        for (i, range) in table.iter_mut().enumerate().rev() {
            if !range.is_full() {
                if i < table.len() - 1 {
                    // move the next available entry to the front
                    let len = table.len();
                    table.swap(i, len - 1);
                }
                available_range = table.last_mut();
                break;
            }
        }
        
        // unwrap the range, or create a new one if none are found
        let available_range = if let Some(range) = available_range {
            range
        } else {
            let base = self.base.plus(RANGE_SIZE * self.size_lookup.len());
            self.size_lookup.push(size);
            table.push(PageRange::new(base, size));
            table.last_mut().unwrap()
        };

        available_range.allocate_next()
    }

    pub fn free(&mut self, address: memory::Address) -> ForthResult {
        self.get_containing_range_mut(address).map(|range| range.free(address))
    }

    pub fn resize(&mut self, address: memory::Address, size: usize) -> Result<memory::Address, Error> {
        let new_size = (size + memory::CELL_SIZE - 1) / memory::CELL_SIZE; 

        let old_size = self.lookup_size(address)?;
        if old_size < new_size {
            self.free(address)?;
            self.allocate(new_size)
        } else {
            Ok(address)
        }
    }

    fn lookup_size(&self, address: memory::Address) -> Result<usize, Error> {
        let index = (address.offset_from(self.base) >> 12) / PAGES_PER_RANGE;
        if index >= self.size_lookup.len() {
            return Err(Error::InvalidAddress)
        }

        Ok(self.size_lookup[index])
    }

    fn get_containing_range_mut(&mut self, address: memory::Address) -> Result<&mut PageRange, Error> {
        let size = self.lookup_size(address)?;
        let (_, table) = self.bins.get_bin_mut(size).get_page_ranges_mut(size)?;

        for range in table.iter_mut().rev() {
            if range.in_range(address) {
                return Ok(range)
            }
        }

        Err(Error::InvalidAddress)
    }

    fn get_containing_range(&self, address: memory::Address) -> Result<&PageRange, Error> {
        let size = self.lookup_size(address)?;
        let (_, table) = self.bins.get_bin(size).get_page_ranges(size)?;

        for range in table.iter().rev() {
            if range.in_range(address) {
                return Ok(range)
            }
        }

        Err(Error::InvalidAddress)
    }
}

impl MemorySegment for Heap {
    fn get_base(&self) -> memory::Address {
        self.base
    }

    fn get_end(&self) -> memory::Address {
        self.base.plus(RANGE_SIZE * self.size_lookup.len())
    }

    fn write_value(&mut self, address: memory::Address, value: Value) -> Result<(), Error> {
        self.get_containing_range_mut(address).map(|range| {
            range.write(address, value)
        })
    }

    fn read_value(&self, address: memory::Address) -> Result<Value, Error> {
        self.get_containing_range(address).map(|range| {
            range.read(address)
        })
    }
}

impl ToString for Heap {
    fn to_string(&self) -> String {
        let mut s = String::new();
        s.push_str(&format!("smallbin [{} {} {}]:\n", self.bins.smallbin.start_size, self.bins.smallbin.end_size, self.bins.smallbin.step));
        for (i, regions) in self.bins.smallbin.sections.iter().enumerate() {
            s.push_str(&format!("\nsize: {}:\n", self.bins.smallbin.start_size + self.bins.smallbin.step * i));
            for region in regions.iter() {
                s.push_str(&format!("\n{}\n", region.base.to_string()));
                s.push_str("available: ");
                for av in region.available.iter() {
                    s.push_str(&format!("{}, ", av.to_string()));
                }
                s.push_str("\n");
                for (i, value) in region.memory.iter().enumerate() {
                    if i % region.chunk_size == 0 {
                        s.push_str("new_chunk: ");
                    } else {
                        s.push_str("           ");
                    }
                    s.push_str(&format!("{}: {}\n", region.base.plus_cell(i).to_string(), value.to_string()));
                }
            }
        }

        return s
    }
}

#[test]
fn basic_allocations_test() {
    let mut heap = Heap::new(0x7feadface000);
    let allocation_1 = heap.allocate(50);
    assert!(allocation_1.is_ok());
    let allocation_2 = heap.allocate(850);
    assert!(allocation_2.is_ok());
    println!("allocation 1 @ {}", allocation_1.unwrap().to_string());
    println!("allocation 2 @ {}", allocation_2.unwrap().to_string());
    println!("heap: {}", heap.to_string());
}

#[test]
fn basic_free_test() {
    let mut heap = Heap::new(0x7feadface000);
    let allocation_1 = heap.allocate(50);
    let allocation_2 = heap.allocate(50);
    let allocation_3 = heap.allocate(50);
    let allocation_4 = heap.allocate(50);
    assert!(allocation_1.is_ok());
    assert!(allocation_2.is_ok());
    assert!(allocation_3.is_ok());
    assert!(allocation_4.is_ok());
    assert!(heap.free(allocation_2.unwrap()).is_ok());
    assert!(heap.free(allocation_4.unwrap()).is_ok());
    assert!(heap.free(allocation_1.unwrap()).is_ok());
    println!("heap: {}", heap.to_string());
}

#[test]
fn read_write_test() {
    let mut state = crate::evaluate::ForthState::new();
    // make and verify allocations
    let allocations = vec![
        state.heap.allocate(50).unwrap(),
        state.heap.allocate(50).unwrap(),
        state.heap.allocate(50).unwrap(),
        state.heap.allocate(50).unwrap(),
        state.heap.allocate(90).unwrap(),
        state.heap.allocate(90).unwrap(),
    ];

    for (i, allocation) in allocations.iter().enumerate() {
        println!("{}: {}", i, allocation.to_string());
    }

    // use allocations
    assert!(state.write::<u64>(allocations[1], 5).is_ok());
    assert!(state.write::<u64>(allocations[1].plus_cell(3), 0xffffffff).is_ok());
    assert!(state.write::<u64>(allocations[5], 6).is_ok());
    assert!(state.write::<u64>(allocations[5].plus_cell(10), 0xabcd).is_ok());
    assert!(state.write::<u64>(allocations[1], 10).is_ok());

    assert_eq!(state.read::<u64>(allocations[1]), Ok(10));
    assert_eq!(state.read::<u64>(allocations[1].plus_cell(3)), Ok(0xffffffff));
    assert_eq!(state.read::<u64>(allocations[5]), Ok(6));
    assert_eq!(state.read::<u64>(allocations[5].plus_cell(10)), Ok(0xabcd));

    println!("heap: {}", state.heap.to_string());

    assert!(false)
}