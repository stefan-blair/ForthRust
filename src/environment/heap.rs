use crate::evaluate::{ForthResult, Error};
use crate::environment::{value::{Value}, generic_numbers::AsValue, units::{Bytes, Cells, Pages}};
use super::memory::{MemorySegment, Address};


const SMALLBIN_SIZE: Cells = Bytes::bytes(0x200).to_cells();
const SMALLBIN_STEP: Cells = Bytes::bytes(0x10).to_cells();
const LARGEBIN_SIZE: Cells = Bytes::bytes(0x1000).to_cells();
const LARGEBIN_STEP: Cells = Bytes::bytes(0x80).to_cells();

const PAGES_PER_RANGE: Pages = Pages::pages(16);

struct PageRange {
    // address of the first page.  must start at some multiple of 0x10000
    base: Address,
    // the array of actual memory
    memory: Vec<Value>,
    // the size of all chunks within this page range, in cells
    chunk_size: Cells,
    // a list of all available chunks that are before the empty tail area
    available: Vec<Address>,
}

impl PageRange {
    fn new(base: Address, chunk_size: Cells) -> Self {
        Self {
            base, chunk_size,
            memory: Vec::new(),
            available: Vec::new()
        }
    }

    fn num_cells(&self) -> Cells {
        Cells::cells(self.memory.len())
    }

    fn is_full(&self) -> bool {
        self.num_cells().to_pages() == PAGES_PER_RANGE && self.available.len() == 0
    }

    fn allocate_next(&mut self) -> Result<Address, Error> {
        if self.available.len() > 0 {
            Ok(self.available.pop().unwrap())
        } else if PAGES_PER_RANGE.to_cells() - self.num_cells() >= self.chunk_size {
            let address = self.base.plus_cell(self.num_cells());
            self.memory.resize((self.num_cells() + self.chunk_size).get_cells(), 0.value());
            Ok(address)
        } else {
            Err(Error::InsufficientMemory)
        }
    }

    fn in_range(&self, address: Address) -> bool {
        address.between(self.base, self.base.plus_cell(self.num_cells()))
    }

    fn free(&mut self, address: Address) {
        if address.offset_from(self.base).to_cells() + self.chunk_size == self.num_cells() {
            self.memory.resize((self.num_cells() - self.chunk_size).get_cells(), 0.value());
        } else {
            self.available.push(address);
        }
    }

    fn cell_offset(&self, address: Address) -> Cells {
        address.offset_from(self.base).to_cells()
    }

    fn write(&mut self, address: Address, value: Value) {
        let offset = self.cell_offset(address).get_cells();
        self.memory[offset] = value
    }

    fn read(&self, address: Address) -> Value {
        let offset = self.cell_offset(address).get_cells();
        self.memory[offset]
    }

    fn write_values(&mut self, address: Address, values: &[Value]) {
        let start = self.cell_offset(address).get_cells();
        let end = self.cell_offset(address.plus_cell(Cells::cells(values.len()))).get_cells();

        let slice = &mut self.memory[start..end];
        slice.copy_from_slice(values);
    }

    fn read_values(&self, address: Address, len: usize) -> Vec<Value> {
        let start = self.cell_offset(address).get_cells();
        let end = self.cell_offset(address.plus_cell(Cells::cells(len))).get_cells();

        let mut results = vec![0.value(); len];
        results.copy_from_slice(&self.memory[start..end]);

        results
    }
}

// all of these sizes are in bytes
struct Bin {
    sections: Vec<Vec<PageRange>>,
    start_size: Cells,
    end_size: Cells,
    step: Cells
}

impl Bin {
    fn new(start_size: Cells, end_size: Cells, step: Cells) -> Self {
        let mut sections = Vec::new();
        for _ in (start_size.get_cells()..end_size.get_cells()).step_by(step.get_cells()) {
            sections.push(Vec::new());
        }

        Self { sections, start_size, end_size, step }
    }

    fn get_page_ranges_mut(&mut self, size: Cells) -> Result<(Cells, &mut Vec<PageRange>), Error> {
        if size < self.start_size || size >= self.end_size {
            return Err(Error::InvalidSize)
        }

        let local_size = ((size + self.step - Cells::one()) / self.step.get_cells()) * self.step.get_cells();

        let index = (local_size - self.start_size) / self.step.get_cells();
        Ok((local_size, &mut self.sections[index.get_cells()]))
    }

    fn get_page_ranges(&self, size: Cells) -> Result<(Cells, &Vec<PageRange>), Error> {
        if size < self.start_size || size >= self.end_size {
            return Err(Error::InvalidSize)
        }

        let local_size = ((size + self.step - Cells::one()) / self.step.get_cells()) * self.step.get_cells();

        let index = (local_size - self.start_size) / self.step.get_cells();
        Ok((local_size, &self.sections[index.get_cells()]))
    }
}

struct Bins {
    smallbin: Bin,
    largebin: Bin,
}

impl Bins {
    fn new() -> Self {
        Self {
            smallbin: Bin::new(Cells::zero(), SMALLBIN_SIZE, SMALLBIN_STEP),
            largebin: Bin::new(SMALLBIN_SIZE, LARGEBIN_SIZE, LARGEBIN_STEP),
        }
    }

    fn get_bin_mut(&mut self, size: Cells) -> &mut Bin {
        if size < SMALLBIN_SIZE {
            &mut self.smallbin
        } else if size < LARGEBIN_SIZE {
            &mut self.largebin
        } else {
            panic!()
        }
    }

    fn get_bin(&self, size: Cells) -> &Bin {
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
    base: Address,
    bins: Bins,
    size_lookup: Vec<Cells>,
}

impl Heap {
    pub fn new(base: usize) -> Self {
        Self {
            base: Address::from_raw(Bytes::bytes(base)), 
            bins: Bins::new(),
            size_lookup: Vec::new(),
        }
    }

    pub fn allocate(&mut self, size: Bytes) -> Result<Address, Error> {
        // get the adjusted size, and corresponding table
        let (size, table) = self.bins.get_bin_mut(size.to_cells()).get_page_ranges_mut(size.to_cells())?;

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
            let base = self.base.plus(PAGES_PER_RANGE.to_bytes() * self.size_lookup.len());
            self.size_lookup.push(size);
            table.push(PageRange::new(base, size));
            table.last_mut().unwrap()
        };

        available_range.allocate_next()
    }

    pub fn free(&mut self, address: Address) -> ForthResult {
        self.get_containing_range_mut(address).map(|range| range.free(address))
    }

    pub fn resize(&mut self, address: Address, size: Bytes) -> Result<Address, Error> {
        let old_size = self.lookup_size(address)?;
        if old_size < size.to_cells() {
            self.free(address)?;
            self.allocate(size)
        } else {
            Ok(address)
        }
    }

    fn lookup_size(&self, address: Address) -> Result<Cells, Error> {
        let index = address.offset_from(self.base).to_pages() / PAGES_PER_RANGE;
        if index >= self.size_lookup.len() {
            return Err(Error::InvalidAddress)
        }

        Ok(self.size_lookup[index])
    }

    fn get_containing_range_mut(&mut self, address: Address) -> Result<&mut PageRange, Error> {
        let size = self.lookup_size(address)?;
        let (_, table) = self.bins.get_bin_mut(size).get_page_ranges_mut(size)?;
        
        for range in table.iter_mut().rev() {
            if range.in_range(address) {
                return Ok(range)
            }
        }

        Err(Error::InvalidAddress)
    }

    fn get_containing_range(&self, address: Address) -> Result<&PageRange, Error> {
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
    fn get_base(&self) -> Address {
        self.base
    }

    fn get_end(&self) -> Address {
        self.base.plus(PAGES_PER_RANGE.to_bytes() * self.size_lookup.len())
    }

    fn write_value(&mut self, address: Address, value: Value) -> Result<(), Error> {        
        self.get_containing_range_mut(address).map(|range| {            
            range.write(address, value)
        })
    }

    fn read_value(&self, address: Address) -> Result<Value, Error> {
        self.get_containing_range(address).map(|range| {
            range.read(address)
        })
    }

    fn write_values(&mut self, address: Address, values: &[Value]) -> ForthResult {
        let range = self.get_containing_range_mut(address)?;
        if range.in_range(address.plus_cell(Cells::cells(values.len()))) {
            Ok(range.write_values(address, values))
        } else {
            Err(Error::InvalidAddress)
        }
    }

    fn read_values(&self, address: Address, len: Cells) -> Result<Vec<Value>, Error> {
        let range = self.get_containing_range(address)?;
        if range.in_range(address.plus_cell(len)) {
            Ok(range.read_values(address, len.get_cells()))
        } else {
            Err(Error::InvalidAddress)
        }
    }
}

impl ToString for Heap {
    fn to_string(&self) -> String {
        let mut s = String::new();
        s.push_str(&format!("smallbin [{} {} {}]:\n", self.bins.smallbin.start_size.to_string(), self.bins.smallbin.end_size.to_string(), self.bins.smallbin.step.to_string()));
        for (i, regions) in self.bins.smallbin.sections.iter().enumerate() {
            s.push_str(&format!("\nsize: {}:\n", (self.bins.smallbin.start_size + self.bins.smallbin.step * i).to_string()));
            for region in regions.iter() {
                s.push_str(&format!("\n{}\n", region.base.to_string()));
                s.push_str("available: ");
                for av in region.available.iter() {
                    s.push_str(&format!("{}, ", av.to_string()));
                }
                s.push_str("\n");
                for (i, value) in region.memory.iter().enumerate() {
                    if i % region.chunk_size.get_cells() == 0 {
                        s.push_str("new_chunk: ");
                    } else {
                        s.push_str("           ");
                    }
                    s.push_str(&format!("{}: {}\n", region.base.plus_cell(Cells::cells(i)).to_string(), value.to_string()));
                }
            }
        }

        return s
    }
}

#[test]
fn basic_allocations_test() {
    let mut heap = Heap::new(0x7feadface000);
    let allocation_1 = heap.allocate(Bytes::bytes(50));
    assert!(allocation_1.is_ok());
    let allocation_2 = heap.allocate(Bytes::bytes(850));
    assert!(allocation_2.is_ok());
    println!("allocation 1 @ {}", allocation_1.unwrap().to_string());
    println!("allocation 2 @ {}", allocation_2.unwrap().to_string());
    println!("heap: {}", heap.to_string());
}

#[test]
fn basic_free_test() {
    let mut heap = Heap::new(0x7feadface000);
    let allocation_1 = heap.allocate(Bytes::bytes(50));
    let allocation_2 = heap.allocate(Bytes::bytes(50));
    let allocation_3 = heap.allocate(Bytes::bytes(50));
    let allocation_4 = heap.allocate(Bytes::bytes(50));
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
    let mut state = crate::evaluate::ForthState::new(Default::default());
    // make and verify allocations
    let allocations = vec![Bytes::bytes(50), Bytes::bytes(50), Bytes::bytes(50), Bytes::bytes(50), Bytes::bytes(90), Bytes::bytes(90)]
        .into_iter().map(|bytes| state.heap.allocate(bytes).unwrap()).collect::<Vec<_>>();

    for (i, allocation) in allocations.iter().enumerate() {
        println!("{}: {}", i, allocation.to_string());
    }

    // use allocations
    assert!(state.write::<u64>(allocations[1], 5).is_ok());
    assert!(state.write::<u64>(allocations[1].plus_cell(Cells::cells(3)), 0xffffffff).is_ok());
    println!("error = {:?} address = {}", state.write::<u64>(allocations[5], 6), allocations[5].to_string());
    assert!(state.write::<u64>(allocations[5], 6).is_ok());
    assert!(state.write::<u64>(allocations[5].plus_cell(Cells::cells(10)), 0xabcd).is_ok());
    assert!(state.write::<u64>(allocations[1], 10).is_ok());

    assert_eq!(state.read::<u64>(allocations[1]), Ok(10));
    assert_eq!(state.read::<u64>(allocations[1].plus_cell(Cells::cells(3))), Ok(0xffffffff));
    assert_eq!(state.read::<u64>(allocations[5]), Ok(6));
    assert_eq!(state.read::<u64>(allocations[5].plus_cell(Cells::cells(10))), Ok(0xabcd));

    println!("heap: {}", state.heap.to_string());
}