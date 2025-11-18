// Copyright (c) Microsoft Corporation
// License: MIT OR Apache-2.0

//! # Sample WDM (Windows Driver Model) Driver
//!
//! This is a minimal WDM driver that demonstrates the essential components needed
//! to create a working legacy kernel-mode driver using the `windows-drivers-rs` crates.
//!
//! # What This Example Demonstrates
//!
//! - **DriverEntry**: The required entry point for all Windows drivers
//! - **DriverUnload**: Cleanup callback for driver unloading
//! - **Debug Printing**: Both raw `DbgPrint` and the `println!` macro
//! - **Memory Allocation**: Using `WdkAllocator` for heap allocations in `no_std`
//! - **String Handling**: Converting UTF-16 UNICODE_STRING to Rust String
//! - **Minimal WDM**: No device creation, just entry and exit
//!
//! # WDM vs. KMDF
//!
//! WDM is the legacy driver model. Compared to KMDF:
//!
//! ## Disadvantages of WDM
//! - **More manual work**: Must handle PnP and Power manually
//! - **More error-prone**: Easy to make mistakes in state management
//! - **More boilerplate**: Lots of repetitive code
//! - **Harder to debug**: Less structured than KMDF
//!
//! ## When WDM Might Be Used
//! - Maintaining existing legacy drivers
//! - Extremely simple drivers with no PnP requirements
//! - Learning low-level Windows driver internals
//!
//! **Recommendation**: Use KMDF for new development when possible.
//!
//! # Key Concepts
//!
//! ## WDM Driver Lifecycle
//!
//! 1. **Load**: Windows loads the driver and calls `DriverEntry`
//! 2. **Initialize**: Driver sets up device objects and dispatch routines
//! 3. **Operation**: Driver handles IRPs (I/O Request Packets)
//! 4. **Unload**: `DriverUnload` callback performs cleanup
//!
//! ## no_std Environment
//!
//! Like KMDF, WDM drivers run in kernel mode without std:
//! - Use `#![no_std]` to disable std
//! - Use `extern crate alloc` for heap allocations
//! - Use `WdkAllocator` as the global allocator
//! - Use `wdk_panic` for panic handling
//!
//! ## Safety Requirements
//!
//! WDM provides even fewer safety guardrails than KMDF:
//! - Manual IRP handling with no framework assistance
//! - Manual PnP and Power state management
//! - Easy to create race conditions and deadlocks
//! - Must carefully manage IRQL at all times
//!
//! # Building This Driver
//!
//! ```bash
//! cargo make
//! ```
//!
//! This builds the driver `.sys` file and driver package.
//!
//! # Installing and Loading
//!
//! 1. **Enable Test Signing** (for development):
//!    ```cmd
//!    bcdedit /set testsigning on
//!    ```
//!    Then reboot.
//!
//! 2. **Load the driver** using `sc` command:
//!    ```cmd
//!    sc create SampleWdmDriver binPath= C:\path\to\sample_wdm_driver.sys type= kernel
//!    sc start SampleWdmDriver
//!    ```
//!
//! 3. **View debug output** using DebugView or WinDbg
//!
//! # Debug Output
//!
//! When loaded:
//! ```text
//! Hello World!
//! WDM Driver Entry Complete! Driver Registry Parameter Key: \REGISTRY\...
//! ```
//!
//! When unloaded:
//! ```text
//! Goodbye World!
//! Driver Exit Complete!
//! ```
//!
//! # Extending This Example
//!
//! To create a functional WDM driver, you would add:
//! - Device object creation (`IoCreateDevice`)
//! - Dispatch routine registration
//! - IRP handling for all major functions
//! - PnP and Power IRP handling
//! - Device interface registration
//! - Resource management
//!
//! # Migration to KMDF
//!
//! For new projects, consider using KMDF instead:
//! - See `sample-kmdf-driver` example
//! - KMDF provides framework assistance for PnP, Power, and I/O
//! - Less code, fewer bugs, easier maintenance
//!
//! # See Also
//!
//! - [WDM Programming Guide](https://learn.microsoft.com/en-us/windows-hardware/drivers/kernel/)
//! - [DriverEntry Documentation](https://learn.microsoft.com/en-us/windows-hardware/drivers/wdf/driverentry-for-wdm-drivers)
//! - [Introduction to WDM](https://learn.microsoft.com/en-us/windows-hardware/drivers/kernel/introduction-to-wdm)

#![no_std]
extern crate alloc;

#[cfg(not(test))]
extern crate wdk_panic;

use alloc::{ffi::CString, slice, string::String};

use wdk::println;
#[cfg(not(test))]
use wdk_alloc::WdkAllocator;
use wdk_sys::{ntddk::DbgPrint, DRIVER_OBJECT, NTSTATUS, PCUNICODE_STRING, STATUS_SUCCESS};

#[cfg(not(test))]
#[global_allocator]
static GLOBAL_ALLOCATOR: WdkAllocator = WdkAllocator;

/// `driver_entry` function required by WDM
///
/// # Panics
/// Can panic from unwraps of `CStrings` used internally
///
/// # Safety
/// Function is unsafe since it dereferences raw pointers passed to it from WDM
// SAFETY: "DriverEntry" is the required symbol name for Windows driver entry points.
// No other function in this compilation unit exports this name, preventing symbol conflicts.
#[unsafe(export_name = "DriverEntry")]
pub unsafe extern "system" fn driver_entry(
    driver: &mut DRIVER_OBJECT,
    registry_path: PCUNICODE_STRING,
) -> NTSTATUS {
    // This is an example of directly using DbgPrint binding to print
    let string = CString::new("Hello World!\n").unwrap();

    // SAFETY: This is safe because `string` is a valid pointer to a null-terminated
    // string (`CString` guarantees null-termination)
    unsafe {
        DbgPrint(c"%s".as_ptr().cast(), string.as_ptr());
    }

    driver.DriverUnload = Some(driver_exit);

    // Translate UTF16 string to rust string
    let registry_path = String::from_utf16_lossy(unsafe {
        slice::from_raw_parts(
            (*registry_path).Buffer,
            (*registry_path).Length as usize / core::mem::size_of_val(&(*(*registry_path).Buffer)),
        )
    });

    // It is much better to use the println macro that has an implementation in
    // wdk::print.rs to call DbgPrint. The println! implementation in
    // wdk::print.rs has the same features as the one in std (ex. format args
    // support).
    println!("WDM Driver Entry Complete! Driver Registry Parameter Key: {registry_path}");

    STATUS_SUCCESS
}

extern "C" fn driver_exit(_driver: *mut DRIVER_OBJECT) {
    println!("Goodbye World!");
    println!("Driver Exit Complete!");
}
