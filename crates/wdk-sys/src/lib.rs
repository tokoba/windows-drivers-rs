// Copyright (c) Microsoft Corporation
// License: MIT OR Apache-2.0

//! Direct bindings to APIs available in the Windows Development Kit (WDK)
//!
//! This crate provides low-level, unsafe FFI bindings to the Windows Driver Kit (WDK) APIs,
//! enabling Rust code to interact with Windows kernel-mode and user-mode driver frameworks.
//!
//! # Overview
//!
//! `wdk-sys` is a foundational crate that exposes raw C API bindings generated from WDK headers
//! using bindgen. It supports three driver models:
//!
//! - **WDM** (Windows Driver Model): Legacy kernel-mode driver model
//! - **KMDF** (Kernel-Mode Driver Framework): Modern kernel-mode framework
//! - **UMDF** (User-Mode Driver Framework): User-mode driver framework
//!
//! The active driver model is selected via the `driver_model__driver_type` configuration,
//! which conditionally compiles the appropriate bindings for your driver type.
//!
//! # Architecture
//!
//! The crate is organized into several key modules:
//!
//! - [`ntddk`]: Native Development Kit APIs (WDM/KMDF only)
//! - [`wdf`]: Windows Driver Framework APIs (KMDF/UMDF only)
//! - [`windows`]: Windows API types for UMDF
//! - Device-specific modules: [`gpio`], [`hid`], [`usb`], [`spb`], [`storage`], [`parallel_ports`]
//!
//! # NTSTATUS Helper Functions
//!
//! This crate provides convenient const functions for working with `NTSTATUS` values:
//!
//! - [`NT_SUCCESS`]: Check if status indicates success or informational result
//! - [`NT_INFORMATION`]: Check if status is informational (0x40000000 - 0x7FFFFFFF)
//! - [`NT_WARNING`]: Check if status is a warning (0x80000000 - 0xBFFFFFFF)
//! - [`NT_ERROR`]: Check if status is an error (0xC0000000 - 0xFFFFFFFF)
//!
//! # Examples
//!
//! ```rust,no_run
//! use wdk_sys::{NT_SUCCESS, STATUS_SUCCESS};
//!
//! let status = STATUS_SUCCESS;
//! if NT_SUCCESS(status) {
//!     // Operation succeeded
//! }
//! ```
//!
//! # Safety
//!
//! This crate provides **unsafe** raw FFI bindings. All functions must be called from `unsafe` blocks,
//! and callers are responsible for:
//!
//! - Maintaining memory safety invariants
//! - Following Windows driver API contracts
//! - Ensuring correct IRQL (Interrupt Request Level) for kernel-mode operations
//! - Managing driver object lifetimes
//!
//! For safe, idiomatic Rust wrappers, see the [`wdk`](https://docs.rs/wdk) crate.
//!
//! # Feature Flags
//!
//! - `gpio`: Enable GPIO (General Purpose I/O) device bindings
//! - `hid`: Enable HID (Human Interface Device) bindings
//! - `parallel-ports`: Enable parallel port bindings
//! - `spb`: Enable SPB (Serial Peripheral Bus) bindings
//! - `storage`: Enable storage device bindings
//! - `usb`: Enable USB device bindings
//! - `test-stubs`: Enable test stubs for unit testing driver code
//!
//! # Configuration
//!
//! The driver model must be configured via `driver_model__driver_type` configuration:
//!
//! ```toml
//! [package.metadata.wdk.driver-model]
//! driver-type = "KMDF"  # or "WDM" or "UMDF"
//! ```

#![no_std]

/// Re-export of procedural macros for internal use by wdk-sys.
///
/// This module provides access to WDK procedural macros that generate
/// FFI bindings and helper code at compile time. It's marked as hidden
/// from public documentation as it's an internal implementation detail.
#[cfg(any(
    driver_model__driver_type = "WDM",
    driver_model__driver_type = "KMDF",
    driver_model__driver_type = "UMDF"
))]
#[doc(hidden)]
pub use wdk_macros as __proc_macros;

#[cfg(any(
    driver_model__driver_type = "WDM",
    driver_model__driver_type = "KMDF",
    driver_model__driver_type = "UMDF"
))]
pub use crate::{constants::*, types::*};

/// Native Development Kit (NTDDK) API bindings for kernel-mode drivers.
///
/// This module contains FFI bindings to functions defined in `ntddk.h`,
/// which is the core header for Windows kernel-mode driver development.
/// It provides access to:
///
/// - Device driver management functions (e.g., `IoCreateDevice`, `IoDeleteDevice`)
/// - I/O request packet (IRP) handling
/// - Memory management (e.g., `ExAllocatePool2`, `ExFreePool`)
/// - Synchronization primitives (spinlocks, events, mutexes)
/// - Interrupt handling
/// - DPC (Deferred Procedure Call) management
/// - Registry access
/// - String manipulation
///
/// # Availability
///
/// Only available for WDM and KMDF driver models.
///
/// # Safety
///
/// All functions in this module are `unsafe` and require careful adherence
/// to Windows driver API contracts, including IRQL restrictions.
///
/// # Example
///
/// ```rust,no_run
/// # use wdk_sys::ntddk::*;
/// # use wdk_sys::*;
/// # unsafe {
/// // Check current IRQL level
/// let current_irql = KeGetCurrentIrql();
/// # }
/// ```
#[cfg(any(driver_model__driver_type = "WDM", driver_model__driver_type = "KMDF"))]
pub mod ntddk;

/// Windows API types and bindings for User-Mode Driver Framework (UMDF).
///
/// This module provides access to Windows API types needed for UMDF drivers,
/// which run in user mode and interact with the Windows API rather than
/// kernel APIs directly.
///
/// # Availability
///
/// Only available for UMDF driver model.
#[cfg(driver_model__driver_type = "UMDF")]
pub mod windows;

/// Windows Driver Framework (WDF) API bindings.
///
/// This module contains FFI bindings to the Windows Driver Framework,
/// which provides an object-oriented abstraction over WDM. WDF includes:
///
/// - **KMDF** (Kernel-Mode Driver Framework): For kernel-mode drivers
/// - **UMDF** (User-Mode Driver Framework): For user-mode drivers
///
/// WDF manages common driver tasks such as:
/// - Device and queue management
/// - I/O request handling with automatic cancellation support
/// - Power and PnP state management
/// - Object lifetime management with reference counting
/// - Request forwarding and completion
///
/// # Availability
///
/// Available for both KMDF and UMDF driver models.
///
/// # Example
///
/// ```rust,no_run
/// # #[cfg(any(driver_model__driver_type = "KMDF", driver_model__driver_type = "UMDF"))]
/// # {
/// # use wdk_sys::wdf::*;
/// # use wdk_sys::*;
/// // WDF function table is accessed through generated bindings
/// # }
/// ```
#[cfg(any(driver_model__driver_type = "KMDF", driver_model__driver_type = "UMDF"))]
pub mod wdf;

/// GPIO (General Purpose I/O) device API bindings.
///
/// This module provides FFI bindings for GPIO device drivers, allowing control
/// of general-purpose input/output pins. GPIO is commonly used for:
///
/// - Digital I/O pin control
/// - Hardware signaling and communication
/// - Sensor and actuator interfaces
/// - Custom hardware integration
///
/// # Feature Flag
///
/// Requires the `gpio` feature to be enabled.
///
/// # Availability
///
/// Available for WDM, KMDF, and UMDF driver models.
#[cfg(all(
    any(
        driver_model__driver_type = "WDM",
        driver_model__driver_type = "KMDF",
        driver_model__driver_type = "UMDF"
    ),
    feature = "gpio"
))]
pub mod gpio;

/// HID (Human Interface Device) API bindings.
///
/// This module provides FFI bindings for HID device drivers. HID is used for:
///
/// - Keyboard and mouse drivers
/// - Game controllers and joysticks
/// - Touch screens and digitizers
/// - Custom HID devices (sensors, buttons, etc.)
///
/// HID provides a standardized protocol for human input devices,
/// eliminating the need for device-specific drivers in many cases.
///
/// # Feature Flag
///
/// Requires the `hid` feature to be enabled.
///
/// # Availability
///
/// Available for WDM, KMDF, and UMDF driver models.
#[cfg(all(
    any(
        driver_model__driver_type = "WDM",
        driver_model__driver_type = "KMDF",
        driver_model__driver_type = "UMDF"
    ),
    feature = "hid"
))]
pub mod hid;

/// Parallel port device API bindings.
///
/// This module provides FFI bindings for parallel port (IEEE 1284) drivers.
/// While legacy technology, parallel ports are still used in:
///
/// - Industrial equipment and legacy hardware
/// - Specialized printers and plotters
/// - Data acquisition devices
/// - Custom hardware interfaces
///
/// # Feature Flag
///
/// Requires the `parallel-ports` feature to be enabled.
///
/// # Availability
///
/// Available for WDM, KMDF, and UMDF driver models.
#[cfg(all(
    any(
        driver_model__driver_type = "WDM",
        driver_model__driver_type = "KMDF",
        driver_model__driver_type = "UMDF"
    ),
    feature = "parallel-ports"
))]
pub mod parallel_ports;

/// SPB (Serial Peripheral Bus) API bindings.
///
/// This module provides FFI bindings for Simple Peripheral Bus drivers,
/// including support for common serial protocols:
///
/// - **I²C** (Inter-Integrated Circuit): Two-wire serial protocol
/// - **SPI** (Serial Peripheral Interface): Synchronous serial protocol
/// - **UART** (Universal Asynchronous Receiver/Transmitter): Asynchronous serial
///
/// SPB is used extensively in:
/// - Sensor communication
/// - Display interfaces
/// - Memory devices (EEPROM, Flash)
/// - Low-speed peripheral communication
///
/// # Feature Flag
///
/// Requires the `spb` feature to be enabled.
///
/// # Availability
///
/// Available for WDM, KMDF, and UMDF driver models.
#[cfg(all(
    any(
        driver_model__driver_type = "WDM",
        driver_model__driver_type = "KMDF",
        driver_model__driver_type = "UMDF"
    ),
    feature = "spb"
))]
pub mod spb;

/// Storage device API bindings.
///
/// This module provides FFI bindings for storage device drivers, including:
///
/// - Disk and volume drivers
/// - File system filter drivers
/// - Storage class drivers
/// - Storage port drivers
/// - Virtual storage devices
///
/// Storage drivers handle:
/// - Block-level I/O operations
/// - SCSI command processing
/// - Partition management
/// - Storage stack integration
///
/// # Feature Flag
///
/// Requires the `storage` feature to be enabled.
///
/// # Availability
///
/// Available for WDM, KMDF, and UMDF driver models.
#[cfg(all(
    any(
        driver_model__driver_type = "WDM",
        driver_model__driver_type = "KMDF",
        driver_model__driver_type = "UMDF"
    ),
    feature = "storage"
))]
pub mod storage;

/// USB (Universal Serial Bus) device API bindings.
///
/// This module provides FFI bindings for USB device drivers, supporting:
///
/// - USB client drivers
/// - USB function drivers
/// - USB filter drivers
/// - USB 1.x, 2.0, 3.x protocols
///
/// USB drivers handle:
/// - Device enumeration and configuration
/// - Pipe management and data transfers
/// - Control, bulk, interrupt, and isochronous transfers
/// - Power management and selective suspend
/// - USB descriptor parsing
///
/// # Feature Flag
///
/// Requires the `usb` feature to be enabled.
///
/// # Availability
///
/// Available for WDM, KMDF, and UMDF driver models.
#[cfg(all(
    any(
        driver_model__driver_type = "WDM",
        driver_model__driver_type = "KMDF",
        driver_model__driver_type = "UMDF"
    ),
    feature = "usb"
))]
pub mod usb;

/// Test stubs for unit testing driver code.
///
/// This module provides stub implementations of WDK symbols required
/// for linking test executables. When writing unit tests for driver code,
/// the test binary needs certain WDK symbols (like `DriverEntry`) to link
/// successfully, even though they're never called during testing.
///
/// # Usage
///
/// Add to your `Cargo.toml`:
///
/// ```toml
/// [dev-dependencies]
/// wdk-sys = { version = "...", features = ["test-stubs"] }
/// ```
///
/// # Feature Flag
///
/// Requires the `test-stubs` feature to be enabled.
///
/// # Safety
///
/// These stubs should **never** be called. They exist only to satisfy
/// the linker during test builds.
#[cfg(feature = "test-stubs")]
pub mod test_stubs;

#[cfg(any(
    driver_model__driver_type = "WDM",
    driver_model__driver_type = "KMDF",
    driver_model__driver_type = "UMDF"
))]
mod constants;
#[cfg(any(
    driver_model__driver_type = "WDM",
    driver_model__driver_type = "KMDF",
    driver_model__driver_type = "UMDF"
))]
mod types;

#[cfg(any(
    driver_model__driver_type = "WDM",
    driver_model__driver_type = "KMDF",
    driver_model__driver_type = "UMDF"
))]
mod macros;

/// Windows linker symbol for floating-point support.
///
/// This symbol (`_fltused`) is required by the Windows linker when any floating-point
/// operations appear in the LLVM IR, even when using soft-float targets that don't
/// generate actual floating-point instructions in the final binary.
///
/// LLVM checks for floating-point instructions at the IR level before optimization,
/// so it sets this symbol requirement even when the target specifies soft-float.
/// This stub satisfies the linker without pulling in actual floating-point support.
///
/// # Safety
///
/// This symbol is safe because:
/// - No actual floating-point instructions exist in the compiled binary
/// - The soft-float target configuration ensures emulation if needed
/// - The symbol is only a linker requirement, not a runtime dependency
// SAFETY: _fltused is a required Windows linker symbol for floating point support.
// No other symbols in this crate export this name, preventing linker conflicts.
#[unsafe(no_mangle)]
pub static _fltused: () = ();

/// Stub for Windows C++ exception handler (version 3).
///
/// This function provides a stub implementation of `__CxxFrameHandler3`, which
/// is a C++ exception handling frame handler required by the Windows linker.
///
/// When `panic = "abort"` is configured (standard for drivers), Rust doesn't
/// use C++ exceptions, but the linker still expects this symbol to exist.
/// This stub is never actually called during normal driver operation.
///
/// # Returns
///
/// Always returns 0. The return value is distinct from `__CxxFrameHandler4`
/// to prevent the linker from folding these functions together, which could
/// confuse binary analysis tools.
///
/// # Safety
///
/// This function should never be called in practice. It exists only to satisfy
/// linker requirements.
///
/// # Related Issue
///
/// See <https://github.com/rust-lang/rust/issues/101134> for discussion about
/// avoiding these stubs.
// FIXME: Is there any way to avoid these stubs? See https://github.com/rust-lang/rust/issues/101134
#[cfg(panic = "abort")]
// SAFETY: __CxxFrameHandler3 is a required Windows C++ exception handler symbol.
// No other symbols in this crate export this name, preventing linker conflicts.
#[unsafe(no_mangle)]
pub const extern "system" fn __CxxFrameHandler3() -> i32 {
    0
}

/// Stub for Windows C++ exception handler (version 4).
///
/// This function provides a stub implementation of `__CxxFrameHandler4`, which
/// is a newer version of the C++ exception handling frame handler required by
/// the Windows linker for some compiler configurations.
///
/// Like `__CxxFrameHandler3`, this is required for linking but never called
/// when using Rust with `panic = "abort"`.
///
/// # Returns
///
/// Always returns 1 (different from `__CxxFrameHandler3`'s return value).
/// This distinction prevents the linker from folding these functions together,
/// which helps binary analysis tools correctly identify the code structure.
///
/// # Safety
///
/// This function should never be called in practice. It exists only to satisfy
/// linker requirements.
#[cfg(panic = "abort")]
// SAFETY: __CxxFrameHandler4 is a required Windows C++ exception handler symbol.
// No other symbols in this crate export this name, preventing linker conflicts.
#[unsafe(no_mangle)]
pub const extern "system" fn __CxxFrameHandler4() -> i32 {
    // This is a stub for the C++ exception handling frame handler. It's never
    // called but it needs to be distinct from __CxxFrameHandler3 to not confuse
    // binary analysis tools. We return a different value to prevent folding.
    1
}

/// Stub for Windows security check handler with exception handling (version 4).
///
/// This function provides a stub implementation of `__GSHandlerCheck_EH4`, which
/// combines buffer security checks (GS) with exception handling for version 4
/// of the Windows exception handling mechanism.
///
/// This symbol is required by the Windows linker when security features are enabled,
/// but it's never called in Rust code using `panic = "abort"`.
///
/// # Returns
///
/// Always returns 2 (different from both `__CxxFrameHandler3` and `__CxxFrameHandler4`).
/// Each handler returns a unique value to prevent function folding and maintain
/// distinct symbols for binary analysis tools.
///
/// # Safety
///
/// This function should never be called in practice. It exists only to satisfy
/// linker requirements.
#[cfg(panic = "abort")]
// SAFETY: __GSHandlerCheck_EH4 is a required Windows C++ exception handler symbol.
// No other symbols in this crate export this name, preventing linker conflicts.
#[unsafe(no_mangle)]
pub const extern "system" fn __GSHandlerCheck_EH4() -> i32 {
    // This is a stub for the C++ exception handling frame handler. It's never
    // called but it needs to be distinct from __CxxFrameHandler3 and
    // __CxxFrameHandler4 to not confuse binary analysis tools. We return a
    // different value to prevent folding.
    2
}

#[cfg(any(
    driver_model__driver_type = "WDM",
    driver_model__driver_type = "KMDF",
    driver_model__driver_type = "UMDF"
))]
#[must_use]
#[allow(non_snake_case)]
/// Evaluates to TRUE if the return value specified by `nt_status` is a success
/// type (0 − 0x3FFFFFFF) or an informational type (0x40000000 − 0x7FFFFFFF).
/// This function is taken from ntdef.h in the WDK.
///
/// See the [NTSTATUS reference](https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-erref/87fba13e-bf06-450e-83b1-9241dc81e781) and
/// [Using NTSTATUS values](https://learn.microsoft.com/en-us/windows-hardware/drivers/kernel/using-ntstatus-values) for details.
pub const fn NT_SUCCESS(nt_status: NTSTATUS) -> bool {
    nt_status >= 0
}

#[cfg(any(
    driver_model__driver_type = "WDM",
    driver_model__driver_type = "KMDF",
    driver_model__driver_type = "UMDF"
))]
#[must_use]
#[allow(non_snake_case)]
#[allow(clippy::cast_sign_loss)]
/// Evaluates to TRUE if the return value specified by `nt_status` is an
/// informational type (0x40000000 − 0x7FFFFFFF). This function is taken from
/// ntdef.h in the WDK.
///
/// See the [NTSTATUS reference](https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-erref/87fba13e-bf06-450e-83b1-9241dc81e781) and
/// [Using NTSTATUS values](https://learn.microsoft.com/en-us/windows-hardware/drivers/kernel/using-ntstatus-values) for details.
pub const fn NT_INFORMATION(nt_status: NTSTATUS) -> bool {
    (nt_status as u32 >> 30) == 1
}

#[cfg(any(
    driver_model__driver_type = "WDM",
    driver_model__driver_type = "KMDF",
    driver_model__driver_type = "UMDF"
))]
#[must_use]
#[allow(non_snake_case)]
#[allow(clippy::cast_sign_loss)]
/// Evaluates to TRUE if the return value specified by `nt_status` is a warning
/// type (0x80000000 − 0xBFFFFFFF).  This function is taken from ntdef.h in the
/// WDK.
///
/// See the [NTSTATUS reference](https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-erref/87fba13e-bf06-450e-83b1-9241dc81e781) and
/// [Using NTSTATUS values](https://learn.microsoft.com/en-us/windows-hardware/drivers/kernel/using-ntstatus-values) for details.
pub const fn NT_WARNING(nt_status: NTSTATUS) -> bool {
    (nt_status as u32 >> 30) == 2
}

#[cfg(any(
    driver_model__driver_type = "WDM",
    driver_model__driver_type = "KMDF",
    driver_model__driver_type = "UMDF"
))]
#[must_use]
#[allow(non_snake_case)]
#[allow(clippy::cast_sign_loss)]
/// Evaluates to TRUE if the return value specified by `nt_status` is an error
/// type (0xC0000000 - 0xFFFFFFFF). This function is taken from ntdef.h in the
/// WDK.
///
/// See the [NTSTATUS reference](https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-erref/87fba13e-bf06-450e-83b1-9241dc81e781) and
/// [Using NTSTATUS values](https://learn.microsoft.com/en-us/windows-hardware/drivers/kernel/using-ntstatus-values) for details.
pub const fn NT_ERROR(nt_status: NTSTATUS) -> bool {
    (nt_status as u32 >> 30) == 3
}

/// Asserts that the current code is running at a pageable IRQL.
///
/// This macro provides a debug assertion that verifies the current code is executing
/// at an IRQL level that allows memory to be paged out (IRQL <= APC_LEVEL).
///
/// # Purpose
///
/// In Windows kernel-mode drivers, code and data can be marked as "pageable",
/// meaning it can be paged out to disk when not in use. However, pageable memory
/// can only be accessed when running at IRQL <= APC_LEVEL. Accessing paged memory
/// at higher IRQLs causes a system crash (BSOD).
///
/// This macro should be placed at the beginning of any function that:
/// - Is marked with `#pragma alloc_text(PAGE, ...)` in C
/// - Accesses pageable memory
/// - Calls other functions that require low IRQL
///
/// # Behavior
///
/// In debug builds, this macro checks that `KeGetCurrentIrql() <= APC_LEVEL`.
/// If the assertion fails, it panics, helping catch IRQL violations during development.
///
/// In release builds (when `debug_assertions` is disabled), the macro has no effect
/// and generates no code, so there's no runtime overhead.
///
/// # Example
///
/// ```rust,no_run
/// # #[cfg(any(driver_model__driver_type = "WDM", driver_model__driver_type = "KMDF"))]
/// # {
/// use wdk_sys::PAGED_CODE;
///
/// extern "system" fn my_pageable_function() {
///     // Assert we're at pageable IRQL
///     PAGED_CODE!();
///
///     // Safe to access paged memory here
///     // ...
/// }
/// # }
/// ```
///
/// # Availability
///
/// Only available for WDM and KMDF driver models (not UMDF, which runs in user mode
/// and doesn't have IRQL restrictions).
///
/// # See Also
///
/// - [`KeGetCurrentIrql`](ntddk::KeGetCurrentIrql): Gets the current IRQL
/// - [`APC_LEVEL`]: The maximum IRQL for pageable code
#[cfg(any(driver_model__driver_type = "WDM", driver_model__driver_type = "KMDF"))]
#[macro_export]
#[allow(non_snake_case)]
macro_rules! PAGED_CODE {
    () => {
        debug_assert!(unsafe { $crate::ntddk::KeGetCurrentIrql() <= $crate::APC_LEVEL as u8 });
    };
}
