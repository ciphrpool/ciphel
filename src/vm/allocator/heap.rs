use std::{borrow::BorrowMut, cell::RefCell, fmt::Debug, ops::RangeFull, rc::Rc};

use nom::Offset;
use num_traits::ToBytes;

const ALIGNMENT: usize = 8;
const HEAP_SIZE: usize = 512;

type Pointer = usize;

#[derive(Debug, Clone)]
pub enum HeapError {
    AllocationError,
    WriteError,
    FreeError,
    InvalidPointer,
    Default,
}

#[derive(Clone)]
pub struct Heap {
    heap: Rc<RefCell<[u8; HEAP_SIZE]>>,
    first_freed_block_offset: Rc<RefCell<usize>>,
}

#[derive(Debug, Clone, PartialEq)]
struct BlockHeader {
    size: u64,
    allocated: bool,
}

impl BlockHeader {
    fn allow(&self, data_size: u64) -> bool {
        let available_size = self.size - 16;
        data_size <= available_size
    }

    fn free_from(size: usize) -> Self {
        Self {
            size: size as u64,
            allocated: false,
        }
    }

    fn allocated_from(size: usize) -> Self {
        Self {
            size: size as u64,
            allocated: true,
        }
    }
    fn to_buf(&self) -> [u8; 8] {
        (if self.allocated {
            self.size | 1u64
        } else {
            self.size & !1
        })
        .to_be_bytes()
    }

    fn read(from: &[u8; 8]) -> Self {
        Self {
            size: u64::from_be_bytes(*from) & 0xFFFF_FFFF_FFFF_FFF8,
            allocated: u64::from_be_bytes(*from) & 1u64 != 0,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
enum BlockData {
    Allocated,
    Free { previous: u64, next: u64 },
}

impl BlockData {
    fn fit(size: usize) -> usize {
        if size < 16 {
            16
        } else {
            size
        }
    }
    fn read_free(previous: &[u8; 8], next: &[u8; 8]) -> Self {
        Self::Free {
            previous: u64::from_be_bytes(*previous),
            next: u64::from_be_bytes(*next),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
struct Block {
    pointer: usize,
    header: BlockHeader,
    data: BlockData,
    footer: BlockHeader,
}

impl Block {
    fn range_header(&self) -> std::ops::Range<usize> {
        self.pointer..self.pointer + 8
    }
    fn range_footer(&self) -> std::ops::Range<usize> {
        self.pointer + self.header.size as usize - 8..self.pointer + self.header.size as usize
    }
    fn range_previous(&self) -> Option<std::ops::Range<usize>> {
        match self.data {
            BlockData::Allocated => None,
            BlockData::Free { .. } => Some(self.pointer + 8..self.pointer + 16),
        }
    }
    fn range_next(&self) -> Option<std::ops::Range<usize>> {
        match self.data {
            BlockData::Allocated => None,
            BlockData::Free { .. } => Some(self.pointer + 16..self.pointer + 24),
        }
    }
    fn range_data(&self) -> Option<std::ops::Range<usize>> {
        match self.data {
            BlockData::Allocated => {
                Some(self.pointer + 8..self.pointer + self.header.size as usize - 8)
            }
            BlockData::Free { .. } => None,
        }
    }
    fn peak_left(&self) -> Option<std::ops::Range<usize>> {
        if self.pointer < 8 {
            None
        } else {
            Some(self.pointer - 8..self.pointer)
        }
    }

    fn previous_free(&self) -> Option<usize> {
        match self.data {
            BlockData::Allocated => None,
            BlockData::Free { previous, .. } => {
                if previous & 1u64 != 0 {
                    None
                } else {
                    Some(previous as usize)
                }
            }
        }
    }
    fn next_free(&self) -> Option<usize> {
        match self.data {
            BlockData::Allocated => None,
            BlockData::Free { next, .. } => {
                if next & 1u64 != 0 {
                    None
                } else {
                    Some(next as usize)
                }
            }
        }
    }
    fn skip(&self) -> usize {
        self.pointer + self.header.size as usize
    }

    fn data_size(&self) -> usize {
        self.header.size as usize - 16
    }

    fn cut_to_allocate(self, data_slice: usize) -> Result<(Self, Option<Self>), HeapError> {
        if self.header.allocated || self.data_size() < data_slice {
            return Err(HeapError::InvalidPointer);
        }
        if self.data_size() - data_slice >= 32 {
            let remaining_size = self.data_size() - (data_slice + 16);
            let first_block = {
                let header = BlockHeader::allocated_from(data_slice + 16);
                let footer = BlockHeader::allocated_from(data_slice + 16);
                Block {
                    pointer: self.pointer,
                    header,
                    data: BlockData::Allocated,
                    footer,
                }
            };
            let second_block = {
                let header = BlockHeader::free_from(remaining_size + 16);
                let footer = BlockHeader::free_from(remaining_size + 16);
                Block {
                    pointer: self.pointer + data_slice + 16,
                    header,
                    data: BlockData::Free {
                        previous: self.previous_free().unwrap_or(1) as u64,
                        next: self.next_free().unwrap_or(1) as u64,
                    },
                    footer,
                }
            };
            Ok((first_block, Some(second_block)))
        } else {
            Ok((
                Block {
                    pointer: self.pointer,
                    header: BlockHeader::allocated_from(self.header.size as usize),
                    data: BlockData::Allocated,
                    footer: BlockHeader::allocated_from(self.header.size as usize),
                },
                None,
            ))
        }
    }

    fn coalesce(left_block: &Block, right_block: &Block) -> Option<Self> {
        if left_block.header.allocated || right_block.header.allocated {
            return None;
        }
        let merged_size = left_block.header.size + right_block.header.size;
        let merged_pointer = if left_block.pointer < right_block.pointer {
            left_block.pointer
        } else {
            right_block.pointer
        };
        let merged_data = if left_block.pointer < right_block.pointer {
            let previous_pointer = left_block.previous_free().unwrap_or(1);
            let next_pointer = right_block.next_free().unwrap_or(1);
            BlockData::Free {
                previous: previous_pointer as u64,
                next: next_pointer as u64,
            }
        } else {
            let previous_pointer = right_block.previous_free().unwrap_or(1);
            let next_pointer = left_block.next_free().unwrap_or(1);
            BlockData::Free {
                previous: previous_pointer as u64,
                next: next_pointer as u64,
            }
        };
        Some(Block {
            pointer: merged_pointer,
            header: BlockHeader {
                size: merged_size,
                allocated: false,
            },
            data: merged_data,
            footer: BlockHeader {
                size: merged_size,
                allocated: false,
            },
        })
    }

    fn from_footer(buffer: &[u8], footer_range: std::ops::Range<usize>) -> Result<Self, HeapError> {
        let end = footer_range.end.clone();
        let Ok(block_end) = TryInto::<&[u8; 8]>::try_into(&buffer[footer_range]) else {
            return Err(HeapError::InvalidPointer);
        };
        let block_footer = BlockHeader::read(block_end);
        if end < block_footer.size as usize {
            return Err(HeapError::InvalidPointer);
        }
        let block_pointer = end - block_footer.size as usize;
        Block::read(buffer, block_pointer)
    }

    fn read(buffer: &[u8], offset: Pointer) -> Result<Self, HeapError> {
        if offset as u64 & 1u64 != 0 {
            return Err(HeapError::InvalidPointer);
        }
        if offset > HEAP_SIZE - 32 {
            return Err(HeapError::InvalidPointer);
        }
        let Ok(block_start) = TryInto::<&[u8; 8]>::try_into(&buffer[offset..offset + 8]) else {
            return Err(HeapError::InvalidPointer);
        };
        let block_header = BlockHeader::read(block_start);
        let block_data = if block_header.allocated {
            BlockData::Allocated
        } else {
            let Ok(data_previous) =
                TryInto::<&[u8; 8]>::try_into(&buffer[offset + 8..offset + 8 + 8])
            else {
                return Err(HeapError::InvalidPointer);
            };
            let Ok(data_next) =
                TryInto::<&[u8; 8]>::try_into(&buffer[offset + 8 + 8..offset + 8 + 8 + 8])
            else {
                return Err(HeapError::InvalidPointer);
            };
            BlockData::read_free(data_previous, data_next)
        };
        let Ok(block_end) = TryInto::<&[u8; 8]>::try_into(
            &buffer[offset + block_header.size as usize - 8..offset + block_header.size as usize],
        ) else {
            return Err(HeapError::InvalidPointer);
        };
        let block_footer = BlockHeader::read(block_end);
        if block_header != block_footer {
            return Err(HeapError::InvalidPointer);
        }
        Ok(Self {
            pointer: offset,
            header: block_header,
            data: block_data,
            footer: block_footer,
        })
    }
}

impl Debug for Heap {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Heap")
            .field("heap", &self.iter_blocks())
            .field("first_freed_block_offset", &self.first_freed_block_offset)
            .finish()
    }
}

impl Heap {
    fn iter_blocks(&self) -> Result<Vec<Block>, HeapError> {
        let binding = self.heap.borrow();
        let borrowed = binding.as_ref();
        let mut res = Vec::default();
        let mut offset = 0;
        while offset < HEAP_SIZE {
            let block = Block::read(borrowed, offset)?;
            offset = block.skip();
            res.push(block);
        }
        Ok(res)
    }

    pub fn new() -> Self {
        let header = HEAP_SIZE as u64;

        let res = Self {
            heap: Rc::new(RefCell::new([0; HEAP_SIZE])),
            first_freed_block_offset: Rc::new(RefCell::new(0)),
        };
        {
            // Store the header and the footer of the heap as freed
            let mut borrowed = res.heap.as_ref().borrow_mut();
            borrowed[0..8].copy_from_slice(&HEAP_SIZE.to_be_bytes());

            // set free block previous and next pointer with invalid pointer
            borrowed[8..16].copy_from_slice(&1u64.to_be_bytes());
            borrowed[16..24].copy_from_slice(&1u64.to_be_bytes());

            borrowed[HEAP_SIZE - 8..HEAP_SIZE].copy_from_slice(&HEAP_SIZE.to_be_bytes());
        }
        res
    }

    fn align(size: usize) -> usize {
        (size + (ALIGNMENT - 1)) & (!(ALIGNMENT - 1))
    }

    fn best_fit(&self, aligned_size: usize) -> Result<Option<Block>, HeapError> {
        let mut fitting_block = None;
        let mut min_fitting_size = HEAP_SIZE as u64 + 1;
        // Iterating over free block
        let binding = self.heap.borrow();
        let borrowed = binding.as_ref();

        let mut offset: usize = self.first_freed_block_offset.as_ref().borrow().clone();
        while offset < HEAP_SIZE {
            let block = Block::read(borrowed, offset)?;
            if block.header.allocated {
                offset = block.skip();
                continue;
            }
            offset = block.next_free().unwrap_or(HEAP_SIZE);
            // The block is free

            if block.header.allow(aligned_size as u64) && block.header.size < min_fitting_size {
                min_fitting_size = block.header.size;
                fitting_block = Some(block);
            }
        }
        Ok(fitting_block)
    }

    fn insert(&self, block: &Block) {
        let mut borrowed = self.heap.as_ref().borrow_mut();
        borrowed[block.range_header()].copy_from_slice(&block.header.to_buf());

        match block.data {
            BlockData::Allocated => {}
            BlockData::Free { previous, next } => {
                borrowed[block.range_previous().unwrap()].copy_from_slice(&previous.to_be_bytes());
                borrowed[block.range_next().unwrap()].copy_from_slice(&next.to_be_bytes());
            }
        }

        borrowed[block.range_footer()].copy_from_slice(&block.header.to_buf());
    }

    pub fn alloc(&self, size: usize) -> Result<Pointer, HeapError> {
        let aligned_size = BlockData::fit(Heap::align(size));

        let Some(block) = self.best_fit(aligned_size)? else {
            return Err(HeapError::AllocationError);
        };
        let previous_free_block = block.previous_free();
        let next_free_block = block.next_free();

        let (block, opt_remaining_block) = block.cut_to_allocate(aligned_size)?;
        self.insert(&block);
        match opt_remaining_block {
            Some(remaining_block) => {
                self.insert(&remaining_block);
                let mut borrowed = self.heap.as_ref().borrow_mut();
                // update previous free block to account the change of next free block
                if let Some(previous_free_block) = previous_free_block {
                    let previous_free_block = Block::read(borrowed.as_ref(), previous_free_block)?;
                    if !previous_free_block.header.allocated {
                        borrowed[previous_free_block.range_next().unwrap()]
                            .copy_from_slice(&remaining_block.pointer.to_be_bytes());
                    }
                } else {
                    let mut borrowed_mut_first_freed_block_offset =
                        self.first_freed_block_offset.as_ref().borrow_mut();
                    *borrowed_mut_first_freed_block_offset = remaining_block.pointer as usize;
                }
                // update next free block to account the change of previous free block
                if let Some(next_free_block) = next_free_block {
                    let next_free_block = Block::read(borrowed.as_ref(), next_free_block)?;
                    if !next_free_block.header.allocated {
                        borrowed[next_free_block.range_previous().unwrap()]
                            .copy_from_slice(&remaining_block.pointer.to_be_bytes());
                    }
                }
            }
            None => {
                let mut borrowed = self.heap.as_ref().borrow_mut();
                // update previous free block to account the change of next free block
                if let Some(previous_free_block) = previous_free_block {
                    let previous_free_block = Block::read(borrowed.as_ref(), previous_free_block)?;

                    if let Some(next_free_block) = next_free_block {
                        let next_free_block = Block::read(borrowed.as_ref(), next_free_block)?;
                        if !next_free_block.header.allocated
                            && !previous_free_block.header.allocated
                        {
                            borrowed[previous_free_block.range_next().unwrap()]
                                .copy_from_slice(&next_free_block.pointer.to_be_bytes());
                        }
                    }
                } else {
                    let mut borrowed_mut_first_freed_block_offset =
                        self.first_freed_block_offset.as_ref().borrow_mut();
                    if let Some(next_free_block) = next_free_block {
                        let next_free_block = Block::read(borrowed.as_ref(), next_free_block)?;
                        *borrowed_mut_first_freed_block_offset = next_free_block.pointer;
                    }
                }
                // update next free block to account the change of previous free block
                if let Some(next_free_block) = next_free_block {
                    let next_free_block = Block::read(borrowed.as_ref(), next_free_block)?;
                    if let Some(previous_free_block) = previous_free_block {
                        let previous_free_block =
                            Block::read(borrowed.as_ref(), previous_free_block)?;
                        if !next_free_block.header.allocated
                            && !previous_free_block.header.allocated
                        {
                            borrowed[next_free_block.range_previous().unwrap()]
                                .copy_from_slice(&previous_free_block.pointer.to_be_bytes());
                        }
                    }
                }
            }
        }

        Ok(block.pointer)
    }

    pub fn free(&self, address: Pointer) -> Result<(), HeapError> {
        let block = {
            let binding = self.heap.borrow();
            let borrowed = binding.as_ref();
            Block::read(borrowed, address)?
        };
        if !block.header.allocated {
            return Err(HeapError::FreeError);
        }
        let block = {
            let binding = self.heap.borrow();
            let borrowed = binding.as_ref();
            let borrowed_first_freed_block_offset = self.first_freed_block_offset.borrow();
            let left_block = match block.peak_left() {
                Some(range) => Block::from_footer(borrowed, range).ok(),
                None => None,
            };

            let right_block = Block::read(borrowed, block.skip()).ok();

            let coalesced_left_block = match left_block {
                Some(left_block) => {
                    if left_block.header.allocated {
                        // search for a previous free block to store in the current block
                        let mut offset = *borrowed_first_freed_block_offset;
                        if offset > left_block.pointer {
                            // no free block before the current block therefore set the current block previous pointer to invalid pointer
                            Block {
                                pointer: block.pointer,
                                header: BlockHeader {
                                    size: block.header.size,
                                    allocated: false,
                                },
                                data: BlockData::Free {
                                    previous: 1u64,
                                    next: 1u64,
                                },
                                footer: BlockHeader {
                                    size: block.header.size,
                                    allocated: false,
                                },
                            }
                        } else {
                            // Update the current block previous with the found free block pointer
                            let mut searched_block = Block::read(borrowed, offset)?;
                            while offset < left_block.pointer {
                                searched_block = Block::read(borrowed, offset)?;
                                if searched_block.header.allocated {
                                    offset = block.skip();
                                } else {
                                    offset = searched_block.next_free().unwrap_or(HEAP_SIZE);
                                }
                            }
                            Block {
                                pointer: block.pointer,
                                header: BlockHeader {
                                    size: block.header.size,
                                    allocated: false,
                                },
                                data: BlockData::Free {
                                    previous: searched_block.pointer as u64,
                                    next: 1u64,
                                },
                                footer: BlockHeader {
                                    size: block.header.size,
                                    allocated: false,
                                },
                            }
                        }
                    } else {
                        // Coalesce left
                        let current_block_free = Block {
                            pointer: block.pointer,
                            header: BlockHeader {
                                size: block.header.size,
                                allocated: false,
                            },
                            data: BlockData::Free {
                                previous: 1u64,
                                next: left_block.next_free().unwrap_or(1usize) as u64,
                            },
                            footer: BlockHeader {
                                size: block.header.size,
                                allocated: false,
                            },
                        };
                        Block::coalesce(&left_block, &current_block_free).unwrap()
                    }
                }
                None => {
                    // No left block set borrowed_mut_first_freed_block_offset to block.pointer and set the current block previous pointer to invalid pointer
                    //*borrowed_mut_first_freed_block_offset = block.pointer as usize;
                    Block {
                        pointer: block.pointer,
                        header: BlockHeader {
                            size: block.header.size,
                            allocated: false,
                        },
                        data: BlockData::Free {
                            previous: 1u64,
                            next: 1u64,
                        },
                        footer: BlockHeader {
                            size: block.header.size,
                            allocated: false,
                        },
                    }
                }
            };

            let coalesced_right_block = match right_block {
                Some(right_block) => {
                    if right_block.header.allocated {
                        if coalesced_left_block.next_free().is_some() {
                            coalesced_left_block
                        } else {
                            let mut offset = *borrowed_first_freed_block_offset;
                            if offset < coalesced_left_block.pointer {
                                while offset < HEAP_SIZE {
                                    let searched_block = Block::read(borrowed.as_ref(), offset)?;
                                    if searched_block.header.allocated {
                                        offset = block.skip();
                                    } else {
                                        offset = searched_block.next_free().unwrap_or(HEAP_SIZE);
                                        if offset > coalesced_left_block.pointer {
                                            break;
                                        }
                                    }
                                }
                            }
                            let next_pointer = Block::read(borrowed, offset)
                                .ok()
                                .map(|next_block| next_block.pointer);
                            // update coalesced_left_block next free pointer with searched_block pointer
                            Block {
                                pointer: coalesced_left_block.pointer.clone(),
                                header: coalesced_left_block.header.clone(),
                                data: BlockData::Free {
                                    previous: coalesced_left_block.previous_free().unwrap_or(1usize)
                                        as u64,
                                    next: next_pointer.unwrap_or(1usize) as u64,
                                },
                                footer: coalesced_left_block.footer.clone(),
                            }
                        }
                    } else {
                        Block::coalesce(&coalesced_left_block, &right_block).unwrap()
                    }
                }
                None => Block {
                    pointer: coalesced_left_block.pointer.clone(),
                    header: coalesced_left_block.header.clone(),
                    data: BlockData::Free {
                        previous: coalesced_left_block.previous_free().unwrap_or(1usize) as u64,
                        next: 1u64,
                    },
                    footer: coalesced_left_block.footer.clone(),
                },
            };

            coalesced_right_block
        };
        self.insert(&block);
        {
            let mut borrowed = self.heap.as_ref().borrow_mut();
            let previous_free_block = block.previous_free();
            let next_free_block = block.next_free();

            // update previous free block to account the change of next free block
            if let Some(previous_free_block) = previous_free_block {
                let previous_free_block = Block::read(borrowed.as_ref(), previous_free_block)?;

                if !previous_free_block.header.allocated {
                    borrowed[previous_free_block.range_next().unwrap()]
                        .copy_from_slice(&block.pointer.to_be_bytes());
                    let mut borrowed_mut_first_freed_block_offset =
                        self.first_freed_block_offset.as_ref().borrow_mut();
                    if previous_free_block.pointer < *borrowed_mut_first_freed_block_offset {
                        *borrowed_mut_first_freed_block_offset = block.pointer
                    }
                }
            } else {
                let mut borrowed_mut_first_freed_block_offset =
                    self.first_freed_block_offset.as_ref().borrow_mut();
                if block.pointer < *borrowed_mut_first_freed_block_offset {
                    *borrowed_mut_first_freed_block_offset = block.pointer
                }
            }
            // update next free block to account the change of previous free block
            if let Some(next_free_block) = next_free_block {
                let next_free_block = Block::read(borrowed.as_ref(), next_free_block)?;
                if !next_free_block.header.allocated {
                    borrowed[next_free_block.range_previous().unwrap()]
                        .copy_from_slice(&block.pointer.to_be_bytes());
                }
            }
        }

        Ok(())
    }

    pub fn read(&self, address: Pointer, size: usize) -> Result<Vec<u8>, HeapError> {
        let block = {
            let binding = self.heap.borrow();
            let borrowed = binding.as_ref();
            Block::read(borrowed, address)?
        };
        if !block.header.allocated {
            return Err(HeapError::InvalidPointer);
        }
        if block.data_size() < size {
            return Err(HeapError::InvalidPointer);
        }
        let res = {
            let binding = self.heap.borrow();
            let borrowed = binding.as_ref();
            let data_range = block.range_data();
            match data_range {
                Some(data_range) => {
                    let (start, end) = (data_range.start, data_range.end);
                    if end < start {
                        None
                    } else {
                        let mut output = Vec::with_capacity(end - start);
                        output.extend_from_slice(&borrowed[start..start + size]);
                        Some(output)
                    }
                }
                None => None,
            }
        };
        let Some(res) = res else {
            return Err(HeapError::InvalidPointer);
        };

        Ok(res)
    }

    pub fn write(&self, address: Pointer, data: &Vec<u8>) -> Result<(), HeapError> {
        let block = {
            let binding = self.heap.borrow();
            let borrowed = binding.as_ref();
            Block::read(borrowed, address)?
        };
        if !block.header.allocated {
            return Err(HeapError::InvalidPointer);
        }
        if block.data_size() < data.len() {
            return Err(HeapError::InvalidPointer);
        }
        {
            let mut borrowed = self.heap.as_ref().borrow_mut();
            let Some(data_range) = block.range_data() else {
                return Err(HeapError::InvalidPointer);
            };
            let (start, end) = (data_range.start, data_range.end);
            if start + data.len() >= end {
                return Err(HeapError::WriteError);
            }
            borrowed[start..start + data.len()].copy_from_slice(&data);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::ast::expressions::data;

    use super::*;

    #[test]
    fn valid_alloc_basic() {
        let heap = Heap::new();

        let address = heap.alloc(8).expect("The allocation should have succeeded");
        assert_eq!(address, 0);

        let address = heap
            .alloc(64)
            .expect("The allocation should have succeeded");

        assert_eq!(address, 32);
    }

    #[test]
    fn valid_alloc_free_in_fragmented() {
        let heap = Heap::new();
        let mut pointers = Vec::default();
        for i in 0..6 {
            let address = heap.alloc(8).expect("The allocation should have succeeded");
            pointers.push(address);
        }
        for (i, addr) in pointers.iter().enumerate() {
            if i % 2 == 0 {
                let _ = heap.free(*addr).expect("Free should have succeeded");
            }
        }
        let address = heap.alloc(8).expect("The allocation should have succeeded");
        assert_eq!(address, 0);
    }

    #[test]
    fn valid_coalescing_left() {
        let heap = Heap::new();
        let mut pointers = [0; 6];
        for i in 0..6 {
            let address = heap.alloc(8).expect("The allocation should have succeeded");
            pointers[i] = address;
        }

        let _ = heap.free(pointers[2]).expect("Free should have succeeded");
        // Coalesce Left
        let _ = heap.free(pointers[3]).expect("Free should have succeeded");
        let address = heap
            .alloc(32)
            .expect("The allocation should have succeeded");
        assert_eq!(address, 64);
    }

    #[test]
    fn valid_coalescing_right() {
        let heap = Heap::new();
        let mut pointers = [0; 6];
        for i in 0..6 {
            let address = heap.alloc(8).expect("The allocation should have succeeded");
            pointers[i] = address;
        }

        let _ = heap.free(pointers[3]).expect("Free should have succeeded");
        // Coalesce Right
        let _ = heap.free(pointers[2]).expect("Free should have succeeded");

        let address = heap
            .alloc(32)
            .expect("The allocation should have succeeded");
        assert_eq!(address, 64);
    }

    #[test]
    fn valid_coalescing_complex() {
        let heap = Heap::new();
        let mut pointers = [0; 10];
        for i in 0..10 {
            let address = heap.alloc(8).expect("The allocation should have succeeded");
            pointers[i] = address;
        }
        for i in 5..10 {
            let _ = heap.free(pointers[i]).expect("Free should have succeeded");
        }
        let address = heap
            .alloc(200)
            .expect("The allocation should have succeeded");
        assert_eq!(address, 160);
    }

    #[test]
    fn robustness_coalescing_complex() {
        let heap = Heap::new();
        let mut pointers = [0; HEAP_SIZE / 32];
        for i in 0..HEAP_SIZE / 32 {
            let address = heap.alloc(8).expect("The allocation should have succeeded");
            pointers[i] = address;
        }
        for i in 5..10 {
            let _ = heap.free(pointers[i]).expect("Free should have succeeded");
        }

        let res = heap.alloc(32 * 5);
        assert!(res.is_err());
        let res = heap.alloc(32 * 3);
        assert!(res.is_ok());
        let res = heap.alloc(32 * 1);
        assert!(res.is_ok());
    }

    #[test]
    fn valid_free() {
        let heap = Heap::new();

        let address = heap.alloc(8).expect("The allocation should have succeeded");
        assert_eq!(address, 0);

        heap.free(address).expect("The free should have succeeded");
        let address = heap.alloc(8).expect("The allocation should have succeeded");
        assert_eq!(address, 0);
    }

    #[test]
    fn robustness_heap_overflow() {
        let heap = Heap::new();
        let address = heap.alloc(HEAP_SIZE);
        assert!(address.is_err());
        let address = heap.alloc(HEAP_SIZE - 7);
        assert!(address.is_err());
        let address = heap.alloc(HEAP_SIZE + 1);
        assert!(address.is_err());

        let address = heap
            .alloc(200)
            .expect("The allocation should have succeeded");
        assert_eq!(address, 0);

        let address = heap.alloc(HEAP_SIZE - 200);
        assert!(address.is_err());
    }

    #[test]
    fn robustness_double_free() {
        let heap = Heap::new();

        let address = heap.alloc(8).expect("The allocation should have succeeded");
        assert_eq!(address, 0);

        heap.free(address).expect("The free should have succeeded");
        let res = heap.free(address);
        assert!(res.is_err())
    }

    #[test]
    fn valid_read() {
        let heap = Heap::new();

        let address = heap.alloc(8).expect("The allocation should have succeeded");
        assert_eq!(address, 0);
        let res = heap.read(address, 6).expect("Read should have succeeded");
        assert_eq!(res.len(), 6)
    }

    #[test]
    fn robustness_read() {
        let heap = Heap::new();

        let address = heap.alloc(8).expect("The allocation should have succeeded");
        assert_eq!(address, 0);
        let res = heap.read(address + 1, 6);
        assert!(res.is_err());

        let res = heap.read(address, 30);
        assert!(res.is_err());

        let res = heap.read(HEAP_SIZE + 1, 6);
        assert!(res.is_err());
    }

    #[test]
    fn valid_write() {
        let heap = Heap::new();

        let address = heap.alloc(8).expect("The allocation should have succeeded");
        assert_eq!(address, 0);
        let data = vec![1u8; 6];

        heap.write(address, &data)
            .expect("Write should have succeeded");

        let res = heap.read(address, 6).expect("Read should have succeeded");
        assert_eq!(res, data);
    }

    #[test]
    fn robustness_write() {
        let heap = Heap::new();

        let address = heap.alloc(8).expect("The allocation should have succeeded");
        assert_eq!(address, 0);
        let data = vec![1u8; 64];

        let res = heap.write(address, &data);
        assert!(res.is_err());

        let res = heap.write(address + 1, &data);
        assert!(res.is_err());
    }
}
