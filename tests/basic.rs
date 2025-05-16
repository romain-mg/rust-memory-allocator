use mm_alloc::mm_alloc::*;

#[test]
fn test_malloc_basic() {
    unsafe {
        let ptr = mm_malloc(16);
        assert!(!ptr.is_null(), "malloc returned null on valid input");

        // Check that memory is zeroed
        for i in 0..16 {
            assert_eq!(*ptr.add(i), 0, "allocated memory not zero-filled");
        }

        mm_free(ptr);
    }
}

#[test]
fn test_malloc_2_times_without_initialization() {
    unsafe {
        let ptr_0 = mm_malloc(16);
        mm_free(ptr_0);
        let ptr_1 = mm_malloc(16);
        assert!(!ptr_1.is_null(), "malloc returned null on valid input");

        // Check that memory is zeroed
        for i in 0..16 {
            assert_eq!(*ptr_1.add(i), 0, "allocated memory not zero-filled");
        }
        assert_eq!(ptr_0, ptr_1);
        mm_free(ptr_1);
    }
}

#[test]
fn test_malloc_split_space() {
    unsafe {
        let memory_alloced_in_ptr1 = 8;
        let ptr_0 = mm_malloc(256);
        mm_free(ptr_0);
        let ptr_1 = mm_malloc(memory_alloced_in_ptr1);
        assert!(!ptr_1.is_null(), "malloc returned null on valid input");
        assert_eq!(ptr_0, ptr_1);
        for i in 0..memory_alloced_in_ptr1 {
            assert_eq!(*ptr_1.add(i), 0, "allocated memory not zero-filled");
        }
        let ptr_2 = mm_malloc(8);
        let correct_position_ptr = ptr_1
            .add(memory_alloced_in_ptr1)
            .add(mm_alloc::mm_alloc::SIZE_OF_MEMORY_HEADER);
        assert_eq!(ptr_2, correct_position_ptr);
        for i in 0..8 {
            assert_eq!(*ptr_2.add(i), 0, "allocated memory not zero-filled");
        }
    }
}

#[test]
fn test_malloc_2_times() {
    unsafe {
        let ptr_0 = mm_malloc(16);
        *ptr_0 = 255;
        let ptr_1 = mm_malloc(16);
        assert!(!ptr_1.is_null(), "malloc returned null on valid input");

        // Check that memory is zeroed
        for i in 0..16 {
            assert_eq!(*ptr_1.add(i), 0, "allocated memory not zero-filled");
        }

        mm_free(ptr_0);
        mm_free(ptr_1);
    }
}

#[test]
fn test_malloc_zero_size() {
    unsafe {
        let ptr = mm_malloc(0);
        assert!(ptr.is_null(), "malloc of size 0 should return null");
    }
}

#[test]
fn test_free_null() {
    unsafe {
        mm_free(std::ptr::null_mut()); // should not crash or panic
    }
}

#[test]
fn test_realloc_expand() {
    unsafe {
        let ptr1 = mm_malloc(8);
        assert!(!ptr1.is_null());

        // Fill with values
        for i in 0..8 {
            *ptr1.add(i) = i as u8;
        }

        let ptr2 = mm_realloc(ptr1, 16);
        assert!(!ptr2.is_null());

        // Check preserved values
        for i in 0..8 {
            assert_eq!(*ptr2.add(i), i as u8, "realloc lost data");
        }

        mm_free(ptr2);
    }
}

#[test]
fn test_realloc_to_zero() {
    unsafe {
        let ptr = mm_malloc(10);
        assert!(!ptr.is_null());

        let new_ptr = mm_realloc(ptr, 0);
        assert!(new_ptr.is_null(), "realloc to size 0 should return null");
    }
}

#[test]
fn test_realloc_from_null() {
    unsafe {
        let ptr = mm_realloc(std::ptr::null_mut(), 32);
        assert!(
            !ptr.is_null(),
            "realloc from null should behave like malloc"
        );

        mm_free(ptr);
    }
}

#[test]
fn test_realloc_shrink() {
    unsafe {
        let ptr = mm_malloc(32);
        assert!(!ptr.is_null());

        for i in 0..32 {
            *ptr.add(i) = i as u8;
        }

        let new_ptr = mm_realloc(ptr, 16);
        assert!(!new_ptr.is_null());

        for i in 0..16 {
            assert_eq!(*new_ptr.add(i), i as u8, "shrinking lost data");
        }

        mm_free(new_ptr);
    }
}

#[test]
fn test_free_then_realloc() {
    unsafe {
        let ptr = mm_malloc(24);
        mm_free(ptr);

        let new_ptr = mm_realloc(ptr, 24);
        // It may return the same pointer or a different one, but must be valid.
        assert!(!new_ptr.is_null());
        mm_free(new_ptr);
    }
}

#[test]
fn test_multiple_small_allocations() {
    unsafe {
        let p1 = mm_malloc(8);
        let p2 = mm_malloc(8);
        let p3 = mm_malloc(8);

        assert_ne!(p1, p2);
        assert_ne!(p2, p3);
        assert_ne!(p1, p3);

        mm_free(p1);
        mm_free(p2);
        mm_free(p3);
    }
}

#[test]
fn test_free_coalescing_after_multiple_frees() {
    unsafe {
        let p1 = mm_malloc(16);
        let p2 = mm_malloc(16);
        let p3 = mm_malloc(16);

        mm_free(p1);
        mm_free(p2);
        mm_free(p3);

        let merged = mm_malloc(48);
        assert_eq!(merged, p1, "expected coalesced block to reuse freed space");
        mm_free(merged);
    }
}
