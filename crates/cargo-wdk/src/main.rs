// Copyright (c) Microsoft Corporation
// License: MIT OR Apache-2.0

//! Cargo extension for building and packaging Windows drivers.
//!
//! `cargo-wdk` is a Cargo subcommand that extends Cargo with commands specifically
//! designed for Windows Driver Kit (WDK) development. It provides an integrated
//! workflow for building, packaging, and creating new driver projects.
//!
//! # Overview
//!
//! `cargo-wdk` simplifies Windows driver development by:
//! - **Building drivers**: Compiles WDM, KMDF, and UMDF drivers with correct toolchain settings
//! - **Creating packages**: Generates complete driver packages with INF files and manifests
//! - **Project scaffolding**: Creates new driver projects from templates
//! - **Multi-architecture support**: Builds for AMD64 and ARM64 architectures
//! - **Workspace support**: Handles complex workspace configurations
//!
//! # Installation
//!
//! Install as a Cargo subcommand:
//!
//! ```bash
//! cargo install cargo-wdk
//! ```
//!
//! Or build from source:
//!
//! ```bash
//! cargo install --path crates/cargo-wdk
//! ```
//!
//! # Commands
//!
//! ## `cargo wdk build`
//!
//! Builds a Windows driver and creates a driver package.
//!
//! **Basic usage:**
//! ```bash
//! cargo wdk build
//! ```
//!
//! **Build for specific architecture:**
//! ```bash
//! cargo wdk build --target x86_64-pc-windows-msvc
//! cargo wdk build --target aarch64-pc-windows-msvc
//! ```
//!
//! **Release build:**
//! ```bash
//! cargo wdk build --release
//! ```
//!
//! **Build with custom cargo flags:**
//! ```bash
//! cargo wdk build --verbose
//! cargo wdk build -- --features my-feature
//! ```
//!
//! ## `cargo wdk new`
//!
//! Creates a new Windows driver project from a template.
//!
//! **Create a KMDF driver:**
//! ```bash
//! cargo wdk new my-driver --driver-model kmdf
//! ```
//!
//! **Create a UMDF driver:**
//! ```bash
//! cargo wdk new my-driver --driver-model umdf
//! ```
//!
//! **Create a WDM driver:**
//! ```bash
//! cargo wdk new my-driver --driver-model wdm
//! ```
//!
//! # Project Structure
//!
//! A typical driver project created by `cargo wdk new` has:
//!
//! ```text
//! my-driver/
//! ├── Cargo.toml         # Rust package manifest with WDK metadata
//! ├── build.rs           # Build script calling wdk-build
//! ├── src/
//! │   └── lib.rs         # Driver entry point
//! └── driver-package/    # Driver package files (generated during build)
//!     ├── my-driver.inf  # Driver installation file
//!     ├── my-driver.sys  # Compiled driver binary
//!     └── ...            # Additional package files
//! ```
//!
//! # Configuration
//!
//! Driver configuration is specified in `Cargo.toml`:
//!
//! ```toml
//! [package.metadata.wdk]
//! driver-type = "KMDF"
//! kmdf-version-major = 1
//! target-kmdf-version-minor = 33
//! ```
//!
//! # Build Process
//!
//! When you run `cargo wdk build`, it:
//!
//! 1. **Detects configuration** from `Cargo.toml`
//! 2. **Configures toolchain** via `wdk-build`
//! 3. **Compiles driver** using `cargo build`
//! 4. **Generates INF file** from template
//! 5. **Creates driver package** with all necessary files
//! 6. **Produces output** in `target/` directory
//!
//! # Output Structure
//!
//! Build artifacts are organized as:
//!
//! ```text
//! target/
//! ├── x86_64-pc-windows-msvc/
//! │   ├── debug/
//! │   │   ├── my_driver.dll          # Driver binary (UMDF)
//! │   │   ├── my_driver.sys          # Driver binary (WDM/KMDF)
//! │   │   └── package/               # Complete driver package
//! │   │       ├── my_driver.inf
//! │   │       ├── my_driver.sys
//! │   │       └── my_driver.cat      # Catalog file (if signed)
//! │   └── release/
//! │       └── ...
//! └── aarch64-pc-windows-msvc/
//!     └── ...
//! ```
//!
//! # Environment Variables
//!
//! - `WDKContentRoot`: Path to WDK installation (auto-detected if not set)
//! - `CARGO_WDK_VERBOSE`: Enable verbose logging (0-5)
//!
//! # Requirements
//!
//! - Windows Driver Kit (WDK) installed
//! - Rust with MSVC toolchain
//! - Visual Studio Build Tools (for linker)
//!
//! # Examples
//!
//! ## Create and build a KMDF driver:
//!
//! ```bash
//! # Create new KMDF driver project
//! cargo wdk new my-driver --driver-model kmdf
//!
//! # Navigate to project
//! cd my-driver
//!
//! # Build the driver
//! cargo wdk build
//!
//! # Build release version
//! cargo wdk build --release
//! ```
//!
//! ## Build for multiple architectures:
//!
//! ```bash
//! # Build for AMD64
//! cargo wdk build --target x86_64-pc-windows-msvc
//!
//! # Build for ARM64
//! cargo wdk build --target aarch64-pc-windows-msvc
//! ```
//!
//! # Troubleshooting
//!
//! **WDK not found:**
//! - Ensure WDK is installed
//! - Set `WDKContentRoot` environment variable
//! - Run from eWDK environment (which sets variables automatically)
//!
//! **Linker errors:**
//! - Verify Visual Studio Build Tools are installed
//! - Check that `rustflags = ["-C", "target-feature=+crt-static"]` is set
//!
//! **Build script errors:**
//! - Ensure `build.rs` calls `wdk_build::configure_wdk_binary_build()`
//! - Verify `Cargo.toml` has correct `[package.metadata.wdk]` section
//!
//! # See Also
//!
//! - [`wdk-build`](https://docs.rs/wdk-build): Build script library
//! - [`wdk-sys`](https://docs.rs/wdk-sys): Raw FFI bindings
//! - [`wdk`](https://docs.rs/wdk): Safe Rust wrappers
//! - [WDK Documentation](https://learn.microsoft.com/en-us/windows-hardware/drivers/)

#![allow(clippy::multiple_crate_versions)]
/// The `regex-syntax` and `regex-automata` crates have multiple version
/// dependencies because of the `matchers` crate. This will be resolved by <https://github.com/tokio-rs/tracing/pull/3219>
mod actions;
mod cli;
mod providers;
mod trace;

use anyhow::{Ok, Result};
use clap::Parser;
use cli::Cli;
use tracing::error;

#[cfg(test)]
mod test_utils;

/// Main function for the [`cargo-wdk`][crate] CLI application.
///
/// The main function parses the CLI input, sets up tracing and executes the
/// command. If an error occurs during execution, it logs the error and exits
/// with a non-zero status code.
///
/// # Returns
///
/// `Result<()>`, which is `Ok` on success or an `anyhow::Error` on failure.
///
/// # Errors
///
/// This function will return an error if tracing initialization fails or if the
/// CLI command execution fails.
fn main() -> Result<()> {
    let cli: Cli = Cli::parse();
    trace::init_tracing(cli.verbose);
    cli.run().inspect_err(|e| error!("{}", e))?;
    Ok(())
}
