// Copyright (c) Microsoft Corporation
// License: MIT OR Apache-2.0

//! Default panic handlers for Windows kernel-mode drivers.
//!
//! This crate provides panic handler implementations for kernel-mode drivers built
//! with the Windows Driver Kit (WDK). Since drivers run in `no_std` environments
//! and panics can compromise system stability, these handlers are minimal.
//!
//! # Overview
//!
//! Rust's panic mechanism normally unwinds the stack and runs destructors, but this
//! is unsuitable for kernel-mode code where panics should be avoided entirely. This
//! crate provides panic handlers that:
//!
//! - Stop execution immediately (infinite loop)
//! - Don't unwind or run destructors
//! - Vary behavior based on build type (debug vs. release)
//!
//! # Panic Handler Behavior
//!
//! ## Debug Builds (`debug_assertions` enabled)
//!
//! In debug builds, the panic handler enters an infinite loop:
//! - Preserves the call stack for debugger inspection
//! - Allows developers to attach a debugger and investigate
//! - No additional information is logged (minimal overhead)
//!
//! ## Release Builds (`debug_assertions` disabled)
//!
//! In release builds, the panic handler also enters an infinite loop:
//! - Currently identical to debug behavior
//! - **FIXME**: Should trigger `KeBugCheckEx` for production diagnostics
//!
//! # Usage
//!
//! Simply add `wdk-panic` as a dependency:
//!
//! ```toml
//! [dependencies]
//! wdk-panic = "0.x"
//! ```
//!
//! The panic handler is automatically registered via the `#[panic_handler]` attribute.
//!
//! # Panic Prevention
//!
//! **Drivers should avoid panicking whenever possible.** Panics in drivers can:
//! - Crash the entire system
//! - Corrupt data
//! - Leave hardware in inconsistent states
//! - Cause security vulnerabilities
//!
//! ## Best Practices
//!
//! 1. **Use `Result` for error handling**: Don't panic on errors
//! 2. **Validate inputs**: Check parameters before use
//! 3. **Avoid `.unwrap()` and `.expect()`**: Use `?` or pattern matching
//! 4. **Use `checked_*` arithmetic**: Prevent overflow panics
//! 5. **Test thoroughly**: Catch panics before production
//!
//! ## Example: Safe Error Handling
//!
//! ```rust,no_run
//! use wdk_sys::{NTSTATUS, STATUS_INSUFFICIENT_RESOURCES};
//!
//! fn allocate_buffer(size: usize) -> Result<Vec<u8>, NTSTATUS> {
//!     if size > MAX_ALLOCATION_SIZE {
//!         // Don't panic - return error
//!         return Err(STATUS_INSUFFICIENT_RESOURCES);
//!     }
//!
//!     let mut buffer = Vec::new();
//!     buffer.try_reserve(size)
//!         .map_err(|_| STATUS_INSUFFICIENT_RESOURCES)?;
//!
//!     Ok(buffer)
//! }
//! ```
//!
//! # Future Improvements
//!
//! In the future, the release panic handler may:
//! - Call `KeBugCheckEx` with a driver-specific bug check code
//! - Log panic information to Event Tracing for Windows (ETW)
//! - Capture minidumps for post-mortem analysis
//! - Provide stack traces in debug info
//!
//! # Testing
//!
//! The panic handler is disabled during tests (`#[cfg(not(test))]`), allowing
//! standard Rust test infrastructure to catch panics and report failures.
//!
//! # Availability
//!
//! Only available for WDM and KMDF drivers (kernel-mode). UMDF drivers run in
//! user mode and use the standard Rust panic handler.
//!
//! # See Also
//!
//! - [KeBugCheckEx documentation](https://learn.microsoft.com/en-us/windows-hardware/drivers/ddi/wdm/nf-wdm-kebugcheckex)
//! - [Writing Reliable Drivers](https://learn.microsoft.com/en-us/windows-hardware/drivers/kernel/writing-reliable-drivers)

#![no_std]

#[cfg(not(test))]
use core::panic::PanicInfo;

/// Panic handler for debug builds.
///
/// In debug builds, panics enter an infinite loop to preserve the call stack
/// for debugger inspection. When a debugger is attached, you can:
/// - View the panic location in the call stack
/// - Inspect local variables and driver state
/// - Determine the root cause of the panic
///
/// # Behavior
///
/// - Enters an infinite loop (`loop {}`)
/// - Does not unwind or run destructors
/// - Does not log or format panic messages (minimal overhead)
/// - Preserves register state and call stack
///
/// # Debugging
///
/// When this handler triggers:
/// 1. The system becomes unresponsive (expected)
/// 2. Attach a kernel debugger (WinDbg, etc.)
/// 3. Break into the debugger (`Ctrl+Break` or `Debug` → `Break`)
/// 4. View the call stack with `k` command
/// 5. Inspect variables and identify the panic source
///
/// # Safety
///
/// This function is `const fn` and never returns, meeting the requirements
/// for a `#[panic_handler]`.
#[cfg(all(
    debug_assertions,
    // Disable inclusion of panic handlers when compiling tests for wdk crate
    not(test)
))]
#[panic_handler]
const fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

/// Panic handler for release builds.
///
/// In release builds, panics also enter an infinite loop. This prevents
/// uncontrolled unwinding and potential system corruption.
///
/// # Current Behavior
///
/// Currently identical to debug builds - enters an infinite loop.
///
/// # Future Behavior (FIXME)
///
/// In the future, this should:
/// 1. Call `KeBugCheckEx` with a driver-specific code
/// 2. Pass panic location information if available
/// 3. Trigger crash dump generation for post-mortem analysis
///
/// Example future implementation:
/// ```rust,no_run
/// # use wdk_sys::ntddk::KeBugCheckEx;
/// # const DRIVER_BUGCHECK_CODE: u32 = 0xDEADBEEF;
/// # let (_file, _line, _col) = ("", 0u32, 0u32);
/// // Future implementation:
/// // unsafe {
/// //     KeBugCheckEx(
/// //         DRIVER_BUGCHECK_CODE,
/// //         file_hash,
/// //         line,
/// //         column,
/// //         0
/// //     );
/// // }
/// ```
///
/// # Safety
///
/// This function is `const fn` and never returns, meeting the requirements
/// for a `#[panic_handler]`.
#[cfg(all(
    not(debug_assertions),
    // Disable inclusion of panic handlers when compiling tests for wdk crate
    not(test)
))]
#[panic_handler]
const fn panic(_info: &PanicInfo) -> ! {
    loop {}
    // FIXME: Should this trigger Bugcheck via KeBugCheckEx?
}
