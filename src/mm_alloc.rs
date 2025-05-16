use libc::{c_void, can_err_mask_t, listen, sbrk};
use std::cmp::min;
use std::{
    mem::size_of,
    ptr::{self, copy, null, null_mut},
};

#[repr(C, align(8))]
pub struct MemoryHeader {
    prev: *mut MemoryHeader,
    next: *mut MemoryHeader,
    size: usize,
    free: bool,
    magic: isize,
}

static mut ROOT: *mut MemoryHeader = null_mut();

pub static SIZE_OF_MEMORY_HEADER: usize = size_of::<MemoryHeader>();
static ONE_BYTE: usize = size_of::<u8>();
static MEMORY_ALIGNMENT: usize = 8;
static SIZE_OF_MEMORY_HEADER_IN_BYTES: usize = SIZE_OF_MEMORY_HEADER / ONE_BYTE;

static MAGIC: isize = 0x12345678;

pub unsafe fn mm_malloc(size: usize) -> *mut u8 {
    let mut return_ptr: *mut u8 = null_mut();
    if size == 0 {
        return return_ptr;
    }
    let bits_to_allocate = size + SIZE_OF_MEMORY_HEADER;
    // If no block has been created so far, create one
    if ROOT == null_mut() {
        let raw_root_pointer = sbrk(bits_to_allocate as isize);
        if raw_root_pointer as isize == -1 {
            return return_ptr;
        }
        let root_pointer = raw_root_pointer as *mut MemoryHeader;
        return_ptr = root_pointer.add(1) as *mut u8;
        root_pointer.write(MemoryHeader {
            prev: null_mut(),
            next: null_mut(),
            size: size,
            free: false,
            magic: return_ptr as isize ^ MAGIC,
        });
        ROOT = root_pointer;

    // Else, proceed normally
    } else {
        let mut curr = ROOT;
        let mut prev = null_mut();
        // Walk through the linked list to spot any block that would fit the requested size
        while curr != null_mut() && ((*curr).size < size || !(*curr).free) {
            prev = curr;
            curr = (*curr).next;
        }
        // Allocate new block if we reached the end of the list
        if curr == null_mut() {
            let block_pointer = sbrk(bits_to_allocate as isize) as *mut MemoryHeader;
            if block_pointer as isize == -1 {
                return return_ptr;
            }
            return_ptr = block_pointer.add(1) as *mut u8;
            block_pointer.write(MemoryHeader {
                prev,
                next: null_mut(),
                size: size,
                free: false,
                magic: return_ptr as isize ^ MAGIC,
            });
            (*prev).next = block_pointer;

        // Else, take the first existing fitting block we found
        } else {
            (*curr).free = false;
            return_ptr = curr.add(1) as *mut u8;
            // Split the block into 2 blocks if there is enough space to allocate at least one byte
            let unaligned_memory_block_size = size + SIZE_OF_MEMORY_HEADER;
            let aligned_memory_block_size =
                unaligned_memory_block_size + unaligned_memory_block_size % MEMORY_ALIGNMENT;
            if (*curr).size > size + SIZE_OF_MEMORY_HEADER + ONE_BYTE {
                let current_next = (*curr).next;
                // compute precisely the location of new_next
                let curr_as_u8_ptr = curr as *mut u8;
                let new_next_as_u8_ptr = curr_as_u8_ptr
                    .add(SIZE_OF_MEMORY_HEADER_IN_BYTES)
                    .add(size / ONE_BYTE);
                let new_next = new_next_as_u8_ptr as *mut MemoryHeader;
                new_next.write(MemoryHeader {
                    prev: curr,
                    next: current_next,
                    size: (*curr).size - (size + SIZE_OF_MEMORY_HEADER),
                    free: true,
                    magic: new_next.add(1) as isize ^ MAGIC,
                });
                (*curr).next = new_next;
                (*curr).size -= SIZE_OF_MEMORY_HEADER + (*new_next).size
            }
        }
    }
    // Zeroing the memory
    for i in 0..size {
        let ptr = return_ptr.add(i);
        *ptr = 0;
    }
    return return_ptr;
}
pub unsafe fn mm_free(ptr: *mut u8) {
    // Check if the pointer is valid
    let heap_break = sbrk(0) as *mut u8;
    if ptr.is_null() || ptr > heap_break {
        return;
    }
    let header_ptr = ptr.sub(SIZE_OF_MEMORY_HEADER_IN_BYTES) as *mut MemoryHeader;
    let expected_magic = ptr as isize ^ MAGIC;
    if (*header_ptr).magic != expected_magic {
        return;
    }
    // Free the memory
    let header_ptr = ptr.sub(SIZE_OF_MEMORY_HEADER_IN_BYTES) as *mut MemoryHeader;
    (*header_ptr).free = true;
    // Merge adjacent free blocks
    let mut curr: *mut MemoryHeader;
    let mut next = (*header_ptr).next;
    while !next.is_null() && (*next).free {
        curr = next;
        (*header_ptr).size += (*curr).size + SIZE_OF_MEMORY_HEADER;
        next = (*curr).next;
        (*curr).prev = null_mut();
        (*curr).next = null_mut();
    }

    curr = header_ptr;
    let mut prev = (*curr).prev;
    while !prev.is_null() && (*prev).free {
        (*prev).size += (*curr).size + SIZE_OF_MEMORY_HEADER;
        (*curr).next = null_mut();
        (*curr).prev = null_mut();
        curr = prev;
        prev = (*curr).prev;
    }
}

pub unsafe fn mm_realloc(ptr: *mut u8, size: usize) -> *mut u8 {
    if ptr.is_null() && size == 0 {
        return null_mut();
    } else if ptr.is_null() {
        return mm_malloc(size);
    } else if size == 0 {
        mm_free(ptr);
        return null_mut();
    } else {
        let mut return_ptr = null_mut();
        let ptr_header = ptr.sub(SIZE_OF_MEMORY_HEADER_IN_BYTES) as *mut MemoryHeader;
        let former_size = (*ptr_header).size;
        if size > former_size {
            return_ptr = mm_malloc(size);
            copy(ptr, return_ptr, former_size);
        } else {
            (*ptr_header).size = size;
            return_ptr = ptr;
        }
        return return_ptr;
    }
}
