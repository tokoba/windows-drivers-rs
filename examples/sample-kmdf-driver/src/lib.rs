// Copyright (c) Microsoft Corporation
// License: MIT OR Apache-2.0

//! # Sample KMDF (Kernel-Mode Driver Framework) Driver
//!
//! This is a complete, minimal KMDF driver that demonstrates the essential components
//! needed to create a working kernel-mode driver using the `windows-drivers-rs` crates.
//!
//! # What This Example Demonstrates
//!
//! - **DriverEntry**: The required entry point for all Windows drivers
//! - **WDF Driver Configuration**: Setting up the WDF driver object with callbacks
//! - **Device Creation**: Handling PnP device addition via `EvtDriverDeviceAdd`
//! - **Debug Printing**: Both raw `DbgPrint` and the `println!` macro
//! - **Memory Allocation**: Using `WdkAllocator` for heap allocations
//! - **Panic Handling**: Using `wdk_panic` for `no_std` panic behavior
//! - **String Handling**: Converting UTF-16 UNICODE_STRING to Rust String
//!
//! # Key Concepts
//!
//! ## KMDF Driver Lifecycle
//!
//! 1. **Load**: Windows loads the driver and calls `DriverEntry`
//! 2. **Initialize**: Driver creates WDF driver object with `WdfDriverCreate`
//! 3. **Device Add**: For each device, `EvtDriverDeviceAdd` is called
//! 4. **Operation**: Driver handles I/O and PnP events (not shown in this minimal example)
//! 5. **Unload**: `DriverUnload` callback is invoked during driver cleanup
//!
//! ## no_std Environment
//!
//! KMDF drivers run in kernel mode without access to the standard library:
//! - Use `#![no_std]` to disable std
//! - Use `extern crate alloc` for heap allocations
//! - Use `WdkAllocator` as the global allocator
//! - Use `wdk_panic` for panic handling
//!
//! ## Safety Requirements
//!
//! All WDF function calls are `unsafe` because they:
//! - Dereference raw pointers from Windows
//! - Require following specific API contracts
//! - Must respect IRQL restrictions
//! - Can cause system crashes if misused
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
//! 2. **Install the driver** using Device Manager or `pnputil`:
//!    ```cmd
//!    pnputil /add-driver path\to\driver.inf /install
//!    ```
//!
//! 3. **View debug output** using DebugView or WinDbg
//!
//! # Debug Output
//!
//! When loaded, this driver prints:
//! ```text
//! Hello World!
//! KMDF Driver Entry Complete! Driver Registry Parameter Key: \REGISTRY\...
//! EvtDriverDeviceAdd Entered!
//! WdfDeviceCreate NTSTATUS: 0x00
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
//! To create a functional driver, you would add:
//! - I/O queue creation and callbacks
//! - Device interface registration
//! - Hardware initialization
//! - Interrupt handling
//! - Power management callbacks
//! - PnP state machine handling
//!
//! # See Also
//!
//! - [KMDF Programming Guide](https://learn.microsoft.com/en-us/windows-hardware/drivers/wdf/index)
//! - [DriverEntry Documentation](https://learn.microsoft.com/en-us/windows-hardware/drivers/wdf/driverentry-for-kmdf-drivers)
//! - [WdfDriverCreate](https://learn.microsoft.com/en-us/windows-hardware/drivers/ddi/wdfdriver/nf-wdfdriver-wdfdrivercreate)

#![no_std]

extern crate alloc;

#[cfg(not(test))]
extern crate wdk_panic;

use alloc::{
    ffi::CString,
    slice,
    string::String,
};

use wdk::println;
#[cfg(not(test))]
use wdk_alloc::WdkAllocator;
use wdk_sys::{
    call_unsafe_wdf_function_binding,
    ntddk::DbgPrint,
    DRIVER_OBJECT,
    NTSTATUS,
    PCUNICODE_STRING,
    PDRIVER_OBJECT,
    ULONG,
    UNICODE_STRING,
    WCHAR,
    WDFDEVICE,
    WDFDEVICE_INIT,
    WDFDRIVER,
    WDF_DRIVER_CONFIG,
    WDF_NO_HANDLE,
    WDF_NO_OBJECT_ATTRIBUTES,
};

#[cfg(not(test))]
#[global_allocator]
static GLOBAL_ALLOCATOR: WdkAllocator = WdkAllocator;

/// `DriverEntry` function required by WDF
///
/// # Panics
/// Can panic from unwraps of `CStrings` used internally
///
/// # Safety
/// Function is unsafe since it dereferences raw pointers passed to it from WDF
// SAFETY: "DriverEntry" is the required symbol name for Windows driver entry points.
// No other function in this compilation unit exports this name, preventing symbol conflicts.
#[unsafe(export_name = "DriverEntry")] // WDF expects a symbol with the name DriverEntry
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

    let mut driver_config = {
        let wdf_driver_config_size: ULONG;

        // clippy::cast_possible_truncation cannot currently check compile-time constants: https://github.com/rust-lang/rust-clippy/issues/9613
        #[allow(clippy::cast_possible_truncation)]
        {
            const WDF_DRIVER_CONFIG_SIZE: usize = core::mem::size_of::<WDF_DRIVER_CONFIG>();

            // Manually assert there is not truncation since clippy doesn't work for
            // compile-time constants
            const { assert!(WDF_DRIVER_CONFIG_SIZE <= ULONG::MAX as usize) }

            wdf_driver_config_size = WDF_DRIVER_CONFIG_SIZE as ULONG;
        }

        WDF_DRIVER_CONFIG {
            Size: wdf_driver_config_size,
            EvtDriverDeviceAdd: Some(evt_driver_device_add),
            ..WDF_DRIVER_CONFIG::default()
        }
    };

    let driver_attributes = WDF_NO_OBJECT_ATTRIBUTES;
    let driver_handle_output = WDF_NO_HANDLE.cast::<WDFDRIVER>();

    let wdf_driver_create_ntstatus;
    // SAFETY: This is safe because:
    //         1. `driver` is provided by `DriverEntry` and is never null
    //         2. `registry_path` is provided by `DriverEntry` and is never null
    //         3. `driver_attributes` is allowed to be null
    //         4. `driver_config` is a valid pointer to a valid `WDF_DRIVER_CONFIG`
    //         5. `driver_handle_output` is expected to be null
    unsafe {
        wdf_driver_create_ntstatus = call_unsafe_wdf_function_binding!(
            WdfDriverCreate,
            driver as PDRIVER_OBJECT,
            registry_path,
            driver_attributes,
            &mut driver_config,
            driver_handle_output,
        );
    }

    // Translate UTF16 string to rust string
    let registry_path: UNICODE_STRING =
        // SAFETY: This dereference is safe since `registry_path` is:
        //         * provided by `DriverEntry` and is never null
        //         * a valid pointer to a `UNICODE_STRING`
        unsafe { *registry_path };
    let number_of_slice_elements = {
        registry_path.Length as usize
            / core::mem::size_of_val(
                // SAFETY: This dereference is safe since `Buffer` is:
                //         * provided by `DriverEntry` and is never null
                //         * a valid pointer to `Buffer`'s type
                &unsafe { *registry_path.Buffer },
            )
    };

    let registry_path = String::from_utf16_lossy(
        // SAFETY: This is safe because:
        //         1. `registry_path.Buffer` is valid for reads for `number_of_slice_elements` *
        //            `core::mem::size_of::<WCHAR>()` bytes, and is guaranteed to be aligned and it
        //            must be properly aligned.
        //         2. `registry_path.Buffer` points to `number_of_slice_elements` consecutive
        //            properly initialized values of type `WCHAR`.
        //         3. Windows does not mutate the memory referenced by the returned slice for for
        //            its entire lifetime.
        //         4. The total size, `number_of_slice_elements` * `core::mem::size_of::<WCHAR>()`,
        //            of the slice must be no larger than `isize::MAX`. This is proven by the below
        //            `debug_assert!`.
        unsafe {
            debug_assert!(isize::try_from(
                number_of_slice_elements * core::mem::size_of::<WCHAR>()
            )
            .is_ok());
            slice::from_raw_parts(registry_path.Buffer, number_of_slice_elements)
        },
    );

    // It is much better to use the println macro that has an implementation in
    // wdk::print.rs to call DbgPrint. The println! implementation in
    // wdk::print.rs has the same features as the one in std (ex. format args
    // support).
    println!("KMDF Driver Entry Complete! Driver Registry Parameter Key: {registry_path}");

    wdf_driver_create_ntstatus
}

extern "C" fn evt_driver_device_add(
    _driver: WDFDRIVER,
    mut device_init: *mut WDFDEVICE_INIT,
) -> NTSTATUS {
    println!("EvtDriverDeviceAdd Entered!");

    let mut device_handle_output: WDFDEVICE = WDF_NO_HANDLE.cast();

    let ntstatus;
    // SAFETY: This is safe because:
    //       1. `device_init` is provided by `EvtDriverDeviceAdd` and is never null
    //       2. the argument receiving `WDF_NO_OBJECT_ATTRIBUTES` is allowed to be
    //          null
    //       3. `device_handle_output` is expected to be null
    unsafe {
        ntstatus = call_unsafe_wdf_function_binding!(
            WdfDeviceCreate,
            &mut device_init,
            WDF_NO_OBJECT_ATTRIBUTES,
            &mut device_handle_output,
        );
    }

    println!("WdfDeviceCreate NTSTATUS: {ntstatus:#02x}");
    ntstatus
}

extern "C" fn driver_exit(_driver: *mut DRIVER_OBJECT) {
    println!("Goodbye World!");
    println!("Driver Exit Complete!");
}
