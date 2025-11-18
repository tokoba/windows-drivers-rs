// Copyright (c) Microsoft Corporation
// License: MIT OR Apache-2.0

//! Safe abstractions over Windows Driver Framework (WDF) APIs.
//!
//! This module provides safe, idiomatic Rust wrappers for WDF objects,
//! available in both KMDF (Kernel-Mode Driver Framework) and UMDF
//! (User-Mode Driver Framework).
//!
//! # WDF Object Model
//!
//! WDF uses an object-oriented model where drivers work with framework objects
//! representing devices, queues, requests, and other entities. This module
//! wraps these objects in safe Rust types that:
//!
//! - Automatically manage object lifetimes via RAII
//! - Prevent invalid object states through the type system
//! - Provide ergonomic error handling with `Result`
//! - Enforce thread safety with Send/Sync traits
//!
//! # Available Wrappers
//!
//! ## Synchronization
//!
//! - [`SpinLock`]: Framework spinlock for synchronizing access at DISPATCH_LEVEL
//!
//! ## Timers
//!
//! - [`Timer`]: Framework timer for scheduling deferred procedure calls (DPCs)
//!
//! # Design Principles
//!
//! 1. **Zero-cost abstractions**: Wrappers compile to direct WDF API calls
//! 2. **Memory safety**: Objects can't be used after deletion
//! 3. **Resource safety**: RAII ensures cleanup even on early returns/panics
//! 4. **Type safety**: Incorrect API usage prevented at compile time
//!
//! # Examples
//!
//! ## Creating and Using a Spinlock
//!
//! ```rust,no_run
//! # #[cfg(any(driver_model__driver_type = "KMDF", driver_model__driver_type = "UMDF"))]
//! # {
//! use wdk::wdf::SpinLock;
//! use wdk_sys::WDF_OBJECT_ATTRIBUTES;
//!
//! let mut attrs = WDF_OBJECT_ATTRIBUTES::default();
//! let lock = SpinLock::try_new(&mut attrs)?;
//!
//! // Acquire lock before accessing shared data
//! lock.acquire();
//! // ... critical section ...
//! lock.release();
//! # Ok::<(), wdk_sys::NTSTATUS>(())
//! # }
//! ```
//!
//! ## Creating and Starting a Timer
//!
//! ```rust,no_run
//! # #[cfg(any(driver_model__driver_type = "KMDF", driver_model__driver_type = "UMDF"))]
//! # {
//! use wdk::wdf::Timer;
//! use wdk_sys::{WDF_TIMER_CONFIG, WDF_OBJECT_ATTRIBUTES};
//!
//! let mut timer_config = WDF_TIMER_CONFIG::default();
//! let mut attrs = WDF_OBJECT_ATTRIBUTES::default();
//!
//! let timer = Timer::try_new(&mut timer_config, &mut attrs)?;
//! timer.start(1000000); // Start timer with 1ms interval (100ns units)
//! # Ok::<(), wdk_sys::NTSTATUS>(())
//! # }
//! ```
//!
//! # Safety Considerations
//!
//! While these wrappers are safe to use, WDF programming still requires:
//!
//! - Understanding WDF object hierarchies and parent-child relationships
//! - Respecting IRQL restrictions for each API
//! - Proper callback implementation
//! - Avoiding deadlocks when using multiple locks
//!
//! # See Also
//!
//! - [WDF Documentation](https://learn.microsoft.com/en-us/windows-hardware/drivers/wdf/)
//! - [KMDF Guide](https://learn.microsoft.com/en-us/windows-hardware/drivers/wdf/index)
//! - [UMDF Guide](https://learn.microsoft.com/en-us/windows-hardware/drivers/wdf/getting-started-with-umdf-version-2)

pub use spinlock::*;
pub use timer::*;

mod spinlock;
mod timer;
