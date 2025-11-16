#![no_std]
#![doc = include_str!("../README.md")]

use core::ops::Deref;
use core::ops::DerefMut;

// Serde support (optional feature).
#[cfg(feature = "serde")]
mod serde_impl;

// defmt support (optional feature).
#[cfg(feature = "defmt")]
mod defmt_impl;

/// An array that holds N+EXTRA elements, where N and EXTRA is specified via const generic.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
#[cfg_attr(
    feature = "postcard_max_size",
    derive(postcard::experimental::max_size::MaxSize)
)]
pub struct ArrayPlusExtra<T, const N: usize, const EXTRA: usize> {
    data: [T; N],
    extra: [T; EXTRA],
}

impl<T, const N: usize, const EXTRA: usize> ArrayPlusExtra<T, N, EXTRA>
where
    T: Copy,
{
    /// Create a new array with specified value.
    #[inline]
    pub const fn new(value: T) -> Self {
        Self {
            data: [value; N],
            extra: [value; EXTRA],
        }
    }
}

impl<T, const N: usize, const EXTRA: usize> ArrayPlusExtra<T, N, EXTRA> {
    /// Convert to an array of size `M`. This checks at compile time that `M == N + EXTRA`.
    #[inline]
    pub const fn as_array<const M: usize>(&self) -> &[T; M] {
        const { assert!(M == N + EXTRA) }
        // SAFETY: #[repr(C)] ensures contiguous layout. Compile-time assert guarantees
        // M == N + EXTRA.
        unsafe { core::mem::transmute(self) }
    }

    /// Convert into an owned array of size `M`. This checks at compile time that `M == N + EXTRA`.
    #[inline]
    pub const fn into_array<const M: usize>(self) -> [T; M] {
        const { assert!(M == N + EXTRA) }
        // SAFETY: #[repr(C)] ensures contiguous layout. Compile-time assert guarantees
        // M == N + EXTRA. We manually forget self to not have double drop.
        let this = unsafe { core::ptr::read(&self as *const Self as *const [T; M]) };
        core::mem::forget(self);
        this
    }

    /// Get a slice view of all N+EXTRA elements.
    /// This is a const fn that can be used in const contexts.
    #[inline]
    pub const fn as_slice(&self) -> &[T] {
        // SAFETY: #[repr(C)] guarantees `data` and `extra` are contiguous in memory.
        // Pointer to first element and length N+EXTRA creates a valid slice with correct lifetime.
        unsafe { core::slice::from_raw_parts(self as *const _ as *const T, N + EXTRA) }
    }

    /// Get a mutable slice view of all N+EXTRA elements.
    /// This is a const fn that can be used in const contexts.
    #[inline]
    pub const fn as_mut_slice(&mut self) -> &mut [T] {
        // SAFETY: #[repr(C)] guarantees `data` and `extra` are contiguous in memory.
        // Pointer to first element and length N+EXTRA creates a valid slice with correct lifetime.
        unsafe { core::slice::from_raw_parts_mut(self as *mut _ as *mut T, N + EXTRA) }
    }
}

impl<T, const N: usize, const EXTRA: usize> Deref for ArrayPlusExtra<T, N, EXTRA> {
    type Target = [T];

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        self.as_slice()
    }
}

impl<T, const N: usize, const EXTRA: usize> DerefMut for ArrayPlusExtra<T, N, EXTRA> {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_mut_slice()
    }
}

// Forward PartialEq to slice implementation.
impl<T, const N: usize, const EXTRA: usize> PartialEq for ArrayPlusExtra<T, N, EXTRA>
where
    T: PartialEq,
{
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self[..] == other[..]
    }
}

// Forward Eq to slice implementation.
impl<T, const N: usize, const EXTRA: usize> Eq for ArrayPlusExtra<T, N, EXTRA> where T: Eq {}

// Forward Hash to slice implementation.
impl<T, const N: usize, const EXTRA: usize> core::hash::Hash for ArrayPlusExtra<T, N, EXTRA>
where
    T: core::hash::Hash,
{
    #[inline]
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self[..].hash(state);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Use std for tests only.
    extern crate std;
    use std::format;

    // Tests for EXTRA = 0 (no extra elements).
    #[test]
    fn test_extra_zero_with_n_zero() {
        let arr: ArrayPlusExtra<i32, 0, 0> = ArrayPlusExtra::new(42);
        // Total length should be N + EXTRA = 0 + 0 = 0.
        assert_eq!(arr.len(), 0);
    }

    #[test]
    fn test_extra_zero_with_n_five() {
        let mut arr: ArrayPlusExtra<i32, 5, 0> = ArrayPlusExtra::new(42);
        // Total length should be N + EXTRA = 5 + 0 = 5.
        assert_eq!(arr.len(), 5);
        // Access all N elements through deref - Miri will catch UB.
        for i in 0..5 {
            assert_eq!(arr[i], 42);
        }
        // Mutate elements.
        arr[4] = 100;
        assert_eq!(arr[4], 100);
    }

    // Tests for EXTRA = 1 (classic plus one).
    #[test]
    fn test_extra_one_with_n_zero() {
        let mut arr: ArrayPlusExtra<i32, 0, 1> = ArrayPlusExtra::new(99);
        // Total length should be N + EXTRA = 0 + 1 = 1.
        assert_eq!(arr.len(), 1);
        assert_eq!(arr[0], 99);
        arr[0] = 100;
        assert_eq!(arr[0], 100);
    }

    #[test]
    fn test_extra_one_with_n_five() {
        let arr: ArrayPlusExtra<i32, 5, 1> = ArrayPlusExtra::new(42);
        // Total length should be N + EXTRA = 5 + 1 = 6.
        assert_eq!(arr.len(), 6);
        // Access all N+1 elements through deref - Miri will catch UB.
        for i in 0..6 {
            assert_eq!(arr[i], 42);
        }
    }

    #[test]
    fn test_extra_one_deref_mut() {
        let mut arr: ArrayPlusExtra<i32, 4, 1> = ArrayPlusExtra::new(0);
        // Total length = 4 + 1 = 5.
        assert_eq!(arr.len(), 5);
        // Write to all N+EXTRA elements - Miri will catch if memory layout is wrong.
        for i in 0..5 {
            arr[i] = i as i32;
        }
        // Read them back.
        for i in 0..5 {
            assert_eq!(arr[i], i as i32);
        }
    }

    // Tests for larger EXTRA values.
    #[test]
    fn test_extra_three_with_n_two() {
        let mut arr: ArrayPlusExtra<u32, 2, 3> = ArrayPlusExtra::new(7);
        // Total length should be N + EXTRA = 2 + 3 = 5.
        assert_eq!(arr.len(), 5);
        // Access all elements.
        for i in 0..5 {
            assert_eq!(arr[i], 7);
        }
        // Modify elements across both data and extra regions.
        arr[0] = 10; // In data array.
        arr[1] = 20; // In data array.
        arr[2] = 30; // In extra array (first).
        arr[3] = 40; // In extra array (middle).
        arr[4] = 50; // In extra array (last).

        assert_eq!(arr[0], 10);
        assert_eq!(arr[1], 20);
        assert_eq!(arr[2], 30);
        assert_eq!(arr[3], 40);
        assert_eq!(arr[4], 50);
    }

    #[test]
    fn test_extra_ten_with_n_ten() {
        let mut arr: ArrayPlusExtra<u64, 10, 10> = ArrayPlusExtra::new(0);
        // Total length should be N + EXTRA = 10 + 10 = 20.
        assert_eq!(arr.len(), 20);
        // Interleaved reads and writes to stress test the unsafe code.
        arr[0] = 1;
        arr[9] = 9; // Last of data array.
        arr[10] = 10; // First of extra array.
        arr[19] = 19; // Last of extra array.

        assert_eq!(arr[0], 1);
        assert_eq!(arr[9], 9);
        assert_eq!(arr[10], 10);
        assert_eq!(arr[19], 19);
    }

    #[test]
    fn test_extra_fifty_large() {
        let mut arr: ArrayPlusExtra<u8, 50, 50> = ArrayPlusExtra::new(1);
        // Total length = 50 + 50 = 100.
        assert_eq!(arr.len(), 100);
        // Access first, middle (in data), middle (in extra), and last elements.
        assert_eq!(arr[0], 1);
        assert_eq!(arr[25], 1); // Middle of data.
        assert_eq!(arr[49], 1); // Last of data.
        assert_eq!(arr[50], 1); // First of extra.
        assert_eq!(arr[75], 1); // Middle of extra.
        assert_eq!(arr[99], 1); // Last of extra.

        arr[99] = 255;
        assert_eq!(arr[99], 255);
    }

    // Tests with different types and EXTRA values.
    #[test]
    fn test_different_types_various_extra() {
        // Test with u8, EXTRA=2.
        let arr_u8: ArrayPlusExtra<u8, 2, 2> = ArrayPlusExtra::new(255);
        assert_eq!(arr_u8.len(), 4);
        assert_eq!(arr_u8[3], 255);

        // Test with u64, EXTRA=3.
        let arr_u64: ArrayPlusExtra<u64, 2, 3> = ArrayPlusExtra::new(u64::MAX);
        assert_eq!(arr_u64.len(), 5);
        assert_eq!(arr_u64[4], u64::MAX);

        // Test with f64, EXTRA=5.
        let mut arr_f64: ArrayPlusExtra<f64, 1, 5> = ArrayPlusExtra::new(1.5);
        assert_eq!(arr_f64.len(), 6);
        arr_f64[5] = 2.5;
        assert_eq!(arr_f64[5], 2.5);
    }

    #[test]
    fn test_slice_iteration_various_extra() {
        // EXTRA=1.
        let mut arr: ArrayPlusExtra<i32, 3, 1> = ArrayPlusExtra::new(0);
        assert_eq!(arr.len(), 4);
        arr[0] = 10;
        arr[1] = 20;
        arr[2] = 30;
        arr[3] = 40;

        // Iterate using slice methods - exercises deref.
        let sum: i32 = arr.iter().sum();
        assert_eq!(sum, 100);

        // Mutable iteration - exercises deref_mut.
        for elem in arr.iter_mut() {
            *elem += 1;
        }
        assert_eq!(arr[0], 11);
        assert_eq!(arr[3], 41);

        // EXTRA=5.
        let mut arr2: ArrayPlusExtra<i32, 2, 5> = ArrayPlusExtra::new(1);
        assert_eq!(arr2.len(), 7);
        let sum2: i32 = arr2.iter().sum();
        assert_eq!(sum2, 7);

        for elem in arr2.iter_mut() {
            *elem *= 2;
        }
        for i in 0..7 {
            assert_eq!(arr2[i], 2);
        }
    }

    // Tests for trait implementations.
    #[test]
    fn test_debug() {
        let arr: ArrayPlusExtra<i32, 2, 1> = ArrayPlusExtra::new(42);
        let debug_str = format!("{:?}", arr);
        // Should format like a slice.
        assert!(debug_str.contains("42"));
    }

    #[test]
    #[allow(clippy::clone_on_copy)]
    fn test_clone() {
        let arr: ArrayPlusExtra<i32, 2, 2> = ArrayPlusExtra::new(5);
        let cloned = arr.clone();
        assert_eq!(arr[0], cloned[0]);
        assert_eq!(arr.len(), cloned.len());
        for i in 0..4 {
            assert_eq!(arr[i], cloned[i]);
        }
    }

    #[test]
    fn test_partial_eq() {
        let arr1: ArrayPlusExtra<i32, 2, 1> = ArrayPlusExtra::new(42);
        let arr2: ArrayPlusExtra<i32, 2, 1> = ArrayPlusExtra::new(42);
        let arr3: ArrayPlusExtra<i32, 2, 1> = ArrayPlusExtra::new(99);

        // Equal arrays should be equal.
        assert_eq!(arr1, arr2);
        // Different values should not be equal.
        assert_ne!(arr1, arr3);
    }

    #[test]
    fn test_eq_reflexive() {
        let arr: ArrayPlusExtra<i32, 3, 2> = ArrayPlusExtra::new(7);
        assert_eq!(arr, arr);
    }

    #[test]
    fn test_hash() {
        use core::hash::{Hash, Hasher};
        use std::collections::hash_map::DefaultHasher;

        let arr1: ArrayPlusExtra<i32, 2, 1> = ArrayPlusExtra::new(42);
        let arr2: ArrayPlusExtra<i32, 2, 1> = ArrayPlusExtra::new(42);
        let arr3: ArrayPlusExtra<i32, 2, 1> = ArrayPlusExtra::new(99);

        let mut hasher1 = DefaultHasher::new();
        let mut hasher2 = DefaultHasher::new();
        let mut hasher3 = DefaultHasher::new();

        arr1.hash(&mut hasher1);
        arr2.hash(&mut hasher2);
        arr3.hash(&mut hasher3);

        // Equal values should have equal hashes.
        assert_eq!(hasher1.finish(), hasher2.finish());
        // Different values should (usually) have different hashes.
        assert_ne!(hasher1.finish(), hasher3.finish());
    }

    #[test]
    fn test_copy() {
        let arr: ArrayPlusExtra<i32, 2, 1> = ArrayPlusExtra::new(10);
        let copied = arr; // Should use Copy, not Clone.
        // Original should still be usable.
        assert_eq!(arr[0], 10);
        assert_eq!(copied[0], 10);
    }

    // Tests for const fn methods.
    #[test]
    fn test_as_slice_const_fn() {
        const ARR: ArrayPlusExtra<i32, 2, 1> = ArrayPlusExtra::new(42);
        const SLICE: &[i32] = ARR.as_slice();
        const LEN: usize = SLICE.len();

        assert_eq!(LEN, 3);
        assert_eq!(SLICE[0], 42);
        assert_eq!(SLICE[1], 42);
        assert_eq!(SLICE[2], 42);
    }

    #[test]
    fn test_const_creation_and_slicing() {
        const ARR: ArrayPlusExtra<u8, 5, 3> = ArrayPlusExtra::new(255);
        const SLICE: &[u8] = ARR.as_slice();
        const FIRST: u8 = SLICE[0];
        const LAST_INDEX: usize = SLICE.len() - 1;

        assert_eq!(FIRST, 255);
        assert_eq!(SLICE.len(), 8);
        assert_eq!(SLICE[LAST_INDEX], 255);
    }

    #[test]
    fn test_as_slice_method() {
        let arr: ArrayPlusExtra<i32, 3, 2> = ArrayPlusExtra::new(7);
        let slice = arr.as_slice();

        assert_eq!(slice.len(), 5);
        for &val in slice {
            assert_eq!(val, 7);
        }
    }

    #[test]
    fn test_as_mut_slice_method() {
        let mut arr: ArrayPlusExtra<i32, 2, 2> = ArrayPlusExtra::new(0);
        let slice = arr.as_mut_slice();

        slice[0] = 10;
        slice[1] = 20;
        slice[2] = 30;
        slice[3] = 40;

        assert_eq!(arr[0], 10);
        assert_eq!(arr[1], 20);
        assert_eq!(arr[2], 30);
        assert_eq!(arr[3], 40);
    }

    #[test]
    fn test_const_zero_sized() {
        const ARR: ArrayPlusExtra<i32, 0, 0> = ArrayPlusExtra::new(42);
        const SLICE: &[i32] = ARR.as_slice();
        const LEN: usize = SLICE.len();

        assert_eq!(LEN, 0);
    }

    #[test]
    fn test_as_array() {
        let mut arr: ArrayPlusExtra<i32, 2, 3> = ArrayPlusExtra::new(0);
        arr[0] = 10;
        arr[4] = 50;

        let array_ref: &[i32; 5] = arr.as_array();
        assert_eq!(array_ref[0], 10);
        assert_eq!(array_ref[4], 50);
    }

    #[test]
    fn test_into_array() {
        let mut arr: ArrayPlusExtra<i32, 2, 3> = ArrayPlusExtra::new(0);
        arr[0] = 10;
        arr[4] = 50;

        let array: [i32; 5] = arr.into_array();
        assert_eq!(array[0], 10);
        assert_eq!(array[4], 50);
    }
}
