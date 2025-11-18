// Copyright (c) Microsoft Corporation
// License: MIT OR Apache-2.0

//! Global allocator implementation for Windows kernel-mode drivers.
//!
//! This crate provides a [`GlobalAlloc`] implementation that allows kernel-mode
//! drivers to use Rust's allocation APIs from [`alloc`] and [`std::collections`].
//!
//! # Overview
//!
//! Windows kernel-mode drivers need a custom allocator that uses WDK memory pool
//! APIs (`ExAllocatePool2`/`ExFreePool`) instead of the standard system allocator.
//! This crate provides [`WdkAllocator`], which integrates with Rust's global allocator
//! system to enable seamless use of heap allocations in drivers.
//!
//! # Features
//!
//! - **Non-paged pool allocation**: Uses `POOL_FLAG_NON_PAGED` for DISPATCH_LEVEL safety
//! - **Memory tagging**: All allocations tagged with 'rust' for debugging
//! - **Zero overhead**: Direct mapping to WDK APIs with no additional bookkeeping
//! - **IRQL safe**: Can be used at IRQL <= DISPATCH_LEVEL
//!
//! # Usage
//!
//! Set [`WdkAllocator`] as your global allocator:
//!
//! ```rust, no_run
//! #[cfg(all(
//!     any(driver_model__driver_type = "WDM", driver_model__driver_type = "KMDF"),
//!     not(test)
//! ))]
//! use wdk_alloc::WdkAllocator;
//!
//! #[cfg(all(
//!     any(driver_model__driver_type = "WDM", driver_model__driver_type = "KMDF"),
//!     not(test)
//! ))]
//! #[global_allocator]
//! static GLOBAL_ALLOCATOR: WdkAllocator = WdkAllocator;
//! ```
//!
//! Once set, you can use standard Rust allocation APIs:
//!
//! ```rust,no_run
//! extern crate alloc;
//! use alloc::vec::Vec;
//! use alloc::string::String;
//! use alloc::boxed::Box;
//!
//! # fn driver_function() {
//! // These now work in kernel mode!
//! let mut v = Vec::new();
//! v.push(42);
//!
//! let s = String::from("Hello from kernel mode");
//! let boxed = Box::new(123);
//! # }
//! ```
//!
//! # Memory Pools
//!
//! The allocator uses **non-paged pool** memory:
//! - Always resident in physical memory (never paged out)
//! - Accessible at any IRQL <= DISPATCH_LEVEL
//! - More expensive than paged pool
//! - Limited in size (depleting it can cause system instability)
//!
//! # IRQL Restrictions
//!
//! - ✅ **PASSIVE_LEVEL**: Full support for all operations
//! - ✅ **APC_LEVEL**: Full support for all operations
//! - ✅ **DISPATCH_LEVEL**: Full support for allocate/deallocate
//! - ❌ **>= DIRQL**: Cannot allocate or deallocate
//!
//! # Memory Tagging
//!
//! All allocations are tagged with the 4-byte tag `'rust'` (0x74737572 in memory).
//! This tag:
//! - Appears in WinDbg memory dumps
//! - Helps identify Rust driver allocations
//! - Aids in leak detection and debugging
//!
//! View allocations in WinDbg:
//! ```text
//! !poolfind rust
//! ```
//!
//! # Performance Considerations
//!
//! - **Fast path**: Direct calls to `ExAllocatePool2`/`ExFreePool`
//! - **No overhead**: No additional metadata or bookkeeping
//! - **No alignment support**: Currently allocates with default alignment only
//!
//! # Safety Considerations
//!
//! - Don't exhaust non-paged pool (crashes the system)
//! - Always deallocate unused memory promptly
//! - Be aware allocations never page out (impact on physical memory)
//! - Use paged pool allocations for large, infrequently accessed data (requires custom allocator)
//!
//! # Limitations
//!
//! - **No custom alignment**: Layout alignment is ignored (FIXME)
//! - **Non-paged only**: Cannot allocate from paged pool
//! - **No realloc optimization**: Realloc always allocates new memory and copies
//!
//! # Alternative Allocators
//!
//! For specialized needs, consider:
//! - Paged pool allocations (for data accessed at low IRQL)
//! - Lookaside lists (for frequent small allocations)
//! - Direct `ExAllocatePool2` calls (for fine-grained control)
//!
//! # Availability
//!
//! Only available for WDM and KMDF drivers. UMDF uses the standard system allocator.

#![no_std]

#[cfg(any(driver_model__driver_type = "WDM", driver_model__driver_type = "KMDF"))]
pub use kernel_mode::*;

#[cfg(any(driver_model__driver_type = "WDM", driver_model__driver_type = "KMDF"))]
mod kernel_mode {

    use core::alloc::{GlobalAlloc, Layout};

    use wdk_sys::{
        POOL_FLAG_NON_PAGED,
        SIZE_T,
        ULONG,
        ntddk::{ExAllocatePool2, ExFreePool},
    };

    /// Global allocator for Windows kernel-mode drivers.
    ///
    /// This allocator provides a [`GlobalAlloc`] implementation that uses WDK
    /// non-paged pool memory, enabling use of Rust's standard allocation APIs
    /// in kernel-mode drivers.
    ///
    /// # Memory Pool
    ///
    /// Uses `POOL_FLAG_NON_PAGED` to allocate from non-paged pool:
    /// - Memory is always resident (never paged to disk)
    /// - Accessible at any IRQL <= DISPATCH_LEVEL
    /// - Tagged with `'rust'` (0x74737572) for debugging
    ///
    /// # Usage
    ///
    /// Set as global allocator in your driver:
    ///
    /// ```rust,no_run
    /// # #[cfg(all(any(driver_model__driver_type = "WDM", driver_model__driver_type = "KMDF"), not(test)))]
    /// use wdk_alloc::WdkAllocator;
    ///
    /// # #[cfg(all(any(driver_model__driver_type = "WDM", driver_model__driver_type = "KMDF"), not(test)))]
    /// #[global_allocator]
    /// static GLOBAL_ALLOCATOR: WdkAllocator = WdkAllocator;
    /// ```
    ///
    /// # Safety
    ///
    /// This allocator is only safe to use for allocations at IRQL <= DISPATCH_LEVEL.
    /// Using it at higher IRQLs will cause a system crash.
    ///
    /// The allocator:
    /// - Never panics or unwinds
    /// - Always returns null on allocation failure
    /// - Maintains layout size constraints
    /// - Does NOT support custom alignment (uses default alignment)
    ///
    /// # Limitations
    ///
    /// - **No alignment support**: The `layout.align()` is currently ignored (FIXME)
    /// - **Non-paged only**: Cannot allocate from paged pool
    /// - **No IRQL checking**: Caller must ensure IRQL <= DISPATCH_LEVEL
    ///
    /// # Implementation Details
    ///
    /// - `alloc()`: Calls `ExAllocatePool2(POOL_FLAG_NON_PAGED, size, RUST_TAG)`
    /// - `dealloc()`: Calls `ExFreePool(ptr)`
    /// - Memory tag: `'rust'` = 0x74737572 (little-endian)
    pub struct WdkAllocator;

    // The value of memory tags are stored in little-endian order, so it is
    // convenient to reverse the order for readability in tooling (ie. Windbg)
    const RUST_TAG: ULONG = u32::from_ne_bytes(*b"rust");

    // SAFETY: This is safe because the Wdk allocator:
    //         1. can never unwind since it can never panic
    //         2. has implementations of alloc and dealloc that maintain layout
    //            constraints (FIXME: Alignment of the layout is currently not
    //            supported)
    unsafe impl GlobalAlloc for WdkAllocator {
        unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
            let ptr =
                // SAFETY: `ExAllocatePool2` is safe to call from any `IRQL` <= `DISPATCH_LEVEL` since its allocating from `POOL_FLAG_NON_PAGED`
                unsafe {
                    ExAllocatePool2(POOL_FLAG_NON_PAGED, layout.size() as SIZE_T, RUST_TAG)
                };
            if ptr.is_null() {
                return core::ptr::null_mut();
            }
            ptr.cast()
        }

        unsafe fn dealloc(&self, ptr: *mut u8, _layout: Layout) {
            // SAFETY: `ExFreePool` is safe to call from any `IRQL` <= `DISPATCH_LEVEL`
            // since its freeing memory allocated from `POOL_FLAG_NON_PAGED` in `alloc`
            unsafe {
                ExFreePool(ptr.cast());
            }
        }
    }
}
