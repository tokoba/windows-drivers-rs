// Copyright (c) Microsoft Corporation
// License: MIT OR Apache-2.0

//! Macros for use in the `wdk-sys` crate.
//!
//! This module provides Rust macros that wrap WDK functionality which cannot
//! be directly bound via [bindgen](https://docs.rs/bindgen/latest/bindgen/).
//!
//! # Purpose
//!
//! Many Windows Driver Kit APIs are implemented as:
//! - **Inline functions**: Defined in headers but not exported from libraries
//! - **Preprocessor macros**: C/C++ compile-time code generation
//! - **Function tables**: Indirect function calls through version-specific tables
//!
//! Since bindgen can only generate bindings for exported symbols, these inlined
//! and macro-based APIs require custom Rust implementations.
//!
//! # WDF Function Bindings
//!
//! The primary content of this module is `call_unsafe_wdf_function_binding.rs`,
//! generated at build time. This provides access to the WDF function table,
//! which contains pointers to all WDF API functions.
//!
//! WDF uses an indirect calling convention where:
//! 1. The driver receives a function table during initialization
//! 2. All WDF API calls go through this table
//! 3. The table layout varies by WDF version
//!
//! The generated macros handle this indirection and provide type-safe(r)
//! wrappers around the raw function pointers.
//!
//! # Example
//!
//! ```rust,no_run
//! # #[cfg(any(driver_model__driver_type = "KMDF", driver_model__driver_type = "UMDF"))]
//! # {
//! // WDF function calls use the generated bindings to access the function table
//! // (This is handled transparently by the generated code)
//! # }
//! ```
//!
//! # Generated Code
//!
//! The actual macro implementations are generated during the build process
//! based on the WDK version and driver model configuration.

#[cfg(any(driver_model__driver_type = "KMDF", driver_model__driver_type = "UMDF"))]
mod wdf {
    include!(concat!(
        env!("OUT_DIR"),
        "/call_unsafe_wdf_function_binding.rs"
    ));
}
