// Copyright (c) Microsoft Corporation
// License: MIT OR Apache-2.0

//! Idiomatic Rust wrappers for the Windows Driver Kit (WDK) APIs.
//!
//! This crate is built on top of the raw FFI bindings provided by [`wdk_sys`],
//! and provides a safe, idiomatic Rust interface to the WDK.
//!
//! # Overview
//!
//! The `wdk` crate transforms unsafe WDK C APIs into safe, ergonomic Rust abstractions
//! that follow Rust idioms and leverage the type system to prevent common driver bugs.
//!
//! # Key Features
//!
//! - **Memory Safety**: Prevents use-after-free, double-free, and memory leaks
//! - **Thread Safety**: Uses Rust's Send/Sync traits to enforce proper synchronization
//! - **RAII**: Automatic resource cleanup via Drop trait
//! - **Type Safety**: Strong typing prevents API misuse
//! - **Zero-Cost Abstractions**: Safe wrappers with no runtime overhead
//!
//! # Modules
//!
//! - [`wdf`]: Safe wrappers for Windows Driver Framework (KMDF/UMDF) objects
//!   - [`wdf::SpinLock`]: Thread-safe spinlock for DISPATCH_LEVEL synchronization
//!   - [`wdf::Timer`]: WDF timer for scheduling callbacks
//!
//! # Macros
//!
//! - [`print!`]: Debug print to kernel debugger (like `std::print!`)
//! - [`println!`]: Debug print with newline (like `std::println!`)
//!
//! # Functions
//!
//! - [`dbg_break`]: Trigger debugger breakpoint
//! - [`nt_success`]: Check if NTSTATUS indicates success
//! - [`paged_code`]: Assert code is running at pageable IRQL (KMDF/WDM only)
//!
//! # Design Philosophy
//!
//! This crate follows these principles:
//!
//! 1. **Safety by Default**: Unsafe operations are encapsulated in safe APIs
//! 2. **Minimal Overhead**: Abstractions compile down to direct WDK calls
//! 3. **Idiomatic Rust**: Uses Result, Option, RAII, and other Rust patterns
//! 4. **Incremental Adoption**: Can be mixed with direct `wdk_sys` calls as needed
//!
//! # Example: Simple KMDF Driver
//!
//! ```rust,no_run
//! # #[cfg(any(driver_model__driver_type = "KMDF", driver_model__driver_type = "UMDF"))]
//! # {
//! use wdk::wdf::{SpinLock};
//! use wdk::{println, nt_success};
//! use wdk_sys::{WDF_OBJECT_ATTRIBUTES, WDF_NO_OBJECT_ATTRIBUTES};
//!
//! // Safe spinlock creation
//! let mut attrs = WDF_OBJECT_ATTRIBUTES::default();
//! match SpinLock::try_new(&mut attrs) {
//!     Ok(lock) => {
//!         println!("Spinlock created successfully");
//!         lock.acquire();
//!         // Critical section
//!         lock.release();
//!     }
//!     Err(status) => println!("Failed to create spinlock: {:#x}", status),
//! }
//! # }
//! ```
//!
//! # no_std Support
//!
//! This crate is `no_std` for WDM and KMDF driver models (kernel-mode),
//! but uses `std` for UMDF (user-mode) drivers.
//!
//! # Platform Support
//!
//! - **Architectures**: x86, x86_64, aarch64
//! - **Driver Models**: WDM, KMDF, UMDF
//! - **Windows Versions**: Windows 10 and later
//!
//! # Feature Flags
//!
//! - `alloc`: Enable print macros in kernel-mode (requires global allocator)
//!
//! # Safety Considerations
//!
//! While this crate provides safe wrappers, driver development still requires:
//! - Understanding IRQL restrictions
//! - Proper synchronization for shared state
//! - Correct handling of device power states
//! - Following WDF object hierarchy rules
//! - Avoiding deadlocks and race conditions
//!
//! Always test drivers thoroughly in a controlled environment before deployment.

#![cfg_attr(
    any(driver_model__driver_type = "WDM", driver_model__driver_type = "KMDF"),
    no_std
)]

#[cfg(any(
    all(
        feature = "alloc",
        any(driver_model__driver_type = "WDM", driver_model__driver_type = "KMDF")
    ),
    driver_model__driver_type = "UMDF",
))]
pub use print::_print;
#[cfg(any(
    driver_model__driver_type = "WDM",
    driver_model__driver_type = "KMDF",
    driver_model__driver_type = "UMDF"
))]
pub use wdk_sys::NT_SUCCESS as nt_success;
#[cfg(any(driver_model__driver_type = "WDM", driver_model__driver_type = "KMDF"))]
pub use wdk_sys::PAGED_CODE as paged_code;

#[cfg(any(
    all(
        feature = "alloc",
        any(driver_model__driver_type = "WDM", driver_model__driver_type = "KMDF")
    ),
    driver_model__driver_type = "UMDF",
))]
mod print;

#[cfg(any(driver_model__driver_type = "KMDF", driver_model__driver_type = "UMDF"))]
pub mod wdf;

/// Triggers a breakpoint in the attached debugger.
///
/// This function causes an immediate debugger breakpoint, allowing you to inspect
/// driver state, examine variables, and step through code execution. It's equivalent
/// to the `__debugbreak()` intrinsic in MSVC or `int 3` on x86.
///
/// # Supported Architectures
///
/// - **x86/x86_64**: Uses `int 3` instruction
/// - **aarch64**: Uses `brk #0xF000` instruction
///
/// # Use Cases
///
/// - **Development**: Pause execution at specific code points for inspection
/// - **Debugging**: Investigate unexpected behavior or crashes
/// - **Assertions**: Break when invalid conditions are detected
/// - **Performance**: Profile specific code sections interactively
///
/// # Behavior
///
/// When a debugger is attached:
/// - Execution pauses at the breakpoint
/// - Debugger shows current stack, locals, and registers
/// - Developer can step through code or continue execution
///
/// When no debugger is attached:
/// - **Kernel-mode**: Triggers a bug check (BSOD) - **AVOID IN PRODUCTION**
/// - **User-mode**: Triggers an unhandled exception or crash
///
/// # Examples
///
/// ```rust,no_run
/// use wdk::dbg_break;
///
/// fn investigate_corruption(data: &[u8]) {
///     if data.is_empty() {
///         // Break into debugger to inspect why we got empty data
///         dbg_break();
///     }
/// }
/// ```
///
/// ```rust,no_run
/// use wdk::dbg_break;
///
/// fn debug_only_breakpoint() {
///     #[cfg(debug_assertions)]
///     {
///         // Only break in debug builds
///         dbg_break();
///     }
/// }
/// ```
///
/// # Panics
///
/// Panics if called on an unsupported architecture (neither x86, x86_64, nor aarch64).
///
/// # Safety Notes
///
/// - **Never ship production code with unconditional `dbg_break()` calls**
/// - Always guard with `#[cfg(debug_assertions)]` or runtime checks
/// - Ensure breakpoints are only hit during development/testing
///
/// # Implementation
///
/// Implementations derived from [MSVC `__debugbreak` intrinsic documentation](https://learn.microsoft.com/en-us/cpp/intrinsics/debugbreak?view=msvc-170#remarks)
pub fn dbg_break() {
    // SAFETY: Abides all rules outlined in https://doc.rust-lang.org/reference/inline-assembly.html#rules-for-inline-assembly
    unsafe {
        #[cfg(target_arch = "aarch64")]
        {
            core::arch::asm!("brk #0xF000");
            return;
        }

        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        {
            core::arch::asm!("int 3");
            return;
        }
    }

    #[allow(unreachable_code)] // Code is not dead because of conditional compilation
    {
        panic!("dbg_break function called from unsupported architecture");
    }
}
