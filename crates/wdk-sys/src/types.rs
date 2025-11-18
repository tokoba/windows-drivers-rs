// Copyright (c) Microsoft Corporation
// License: MIT OR Apache-2.0

//! Core type definitions from the Windows Driver Kit (WDK).
//!
//! This module provides FFI-safe type definitions used throughout the WDK,
//! generated from Windows driver headers via bindgen. These types form the
//! foundation for all driver development in this crate.
//!
//! # Type Categories
//!
//! The types in this module include:
//!
//! ## Status and Error Types
//! - `NTSTATUS`: Windows NT status codes for error handling
//! - `HRESULT`: COM-style result codes
//!
//! ## Primitive Windows Types
//! - `UCHAR`, `USHORT`, `ULONG`, `ULONGLONG`: Unsigned integer types
//! - `CHAR`, `SHORT`, `LONG`, `LONGLONG`: Signed integer types
//! - `BOOLEAN`, `BOOL`: Boolean types (different conventions)
//! - `PVOID`, `HANDLE`: Generic pointer and handle types
//!
//! ## String Types
//! - `UNICODE_STRING`, `ANSI_STRING`: Counted string structures
//! - `PWSTR`, `PCWSTR`: Wide (UTF-16) string pointers
//! - `PSTR`, `PCSTR`: ANSI string pointers
//!
//! ## Driver Framework Types
//! - `DRIVER_OBJECT`: Represents a loaded driver
//! - `DEVICE_OBJECT`: Represents a device instance
//! - `IRP`: I/O Request Packet for async I/O operations
//! - `IO_STACK_LOCATION`: IRP stack frame
//!
//! ## WDF Types (KMDF/UMDF)
//! - `WDFDRIVER`, `WDFDEVICE`, `WDFREQUEST`: WDF object handles
//! - `WDF_*_CONFIG`: Configuration structures for WDF objects
//! - `PFN_*`: Function pointer types for callbacks
//!
//! ## Synchronization Types
//! - `KSPIN_LOCK`: Kernel spinlock
//! - `KEVENT`: Kernel event object
//! - `KMUTEX`: Kernel mutex
//! - `ERESOURCE`: Executive resource (reader-writer lock)
//!
//! ## Memory Types
//! - `POOL_TYPE`: Memory pool allocation types
//! - `MDL`: Memory Descriptor List for describing physical memory
//! - `PHYSICAL_ADDRESS`: 64-bit physical memory address
//!
//! # Example
//!
//! ```rust,no_run
//! use wdk_sys::{NTSTATUS, STATUS_SUCCESS, UNICODE_STRING};
//!
//! fn check_status(status: NTSTATUS) -> bool {
//!     status == STATUS_SUCCESS
//! }
//! ```
//!
//! # Safety
//!
//! All types in this module are `repr(C)` and designed for FFI with Windows drivers.
//! When working with these types:
//!
//! - Respect memory layout requirements (padding, alignment)
//! - Follow Windows API initialization requirements (e.g., structure sizes)
//! - Be aware of ownership semantics (many types are reference-counted or managed by Windows)
//! - Understand lifetime requirements for pointer types
//!
//! # Generated Code
//!
//! This module contains auto-generated bindgen code with extensive clippy allow attributes
//! to suppress warnings that are not relevant for generated FFI bindings.

pub use bindings::*;

#[allow(missing_docs)]
#[allow(non_upper_case_globals)]
#[allow(non_camel_case_types)]
#[allow(non_snake_case)]
#[rustversion::attr(
    any(
        all(not(nightly), since(1.88)),
        all(nightly, since(2025-04-25)),
    ),
    allow(unnecessary_transmutes)
)]
#[allow(unsafe_op_in_unsafe_fn)]
#[allow(clippy::cast_lossless)]
#[allow(clippy::cast_possible_truncation)]
#[allow(clippy::cast_possible_wrap)]
#[allow(clippy::cognitive_complexity)]
#[allow(clippy::doc_markdown)]
#[allow(clippy::default_trait_access)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[rustversion::attr(
    any(
        all(not(nightly), before(1.74)),
        all(nightly, before(2023-09-13)),
    ),
    allow(clippy::incorrect_clone_impl_on_copy_type)
)]
#[rustversion::attr(
    any(
        all(not(nightly), since(1.74)),
        all(nightly, since(2023-09-13)),
    ),
    allow(clippy::non_canonical_clone_impl)
)]
#[allow(clippy::missing_const_for_fn)]
#[allow(clippy::missing_safety_doc)]
#[allow(clippy::module_name_repetitions)]
#[allow(clippy::multiple_unsafe_ops_per_block)]
#[allow(clippy::must_use_candidate)]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
#[allow(clippy::ptr_as_ptr)]
#[allow(clippy::ptr_offset_with_cast)]
#[rustversion::attr(
    any(
        all(not(nightly), since(1.77)),
        all(nightly, since(2024-01-11)),
    ),
    allow(clippy::pub_underscore_fields)
)]
#[rustversion::attr(
    any(
        all(not(nightly), since(1.78)),
        all(nightly, since(2024-02-09)),
    ),
    allow(clippy::ref_as_ptr)
)]
#[allow(clippy::semicolon_if_nothing_returned)]
#[allow(clippy::too_many_arguments)]
#[allow(clippy::too_many_lines)]
#[allow(clippy::transmute_ptr_to_ptr)]
#[allow(clippy::undocumented_unsafe_blocks)]
#[allow(clippy::unnecessary_cast)]
#[allow(clippy::unreadable_literal)]
#[allow(clippy::used_underscore_binding)]
#[allow(clippy::useless_transmute)]
#[allow(clippy::use_self)]
mod bindings {
    include!(concat!(env!("OUT_DIR"), "/types.rs"));
}
