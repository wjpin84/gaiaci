//! # Core Module
//! 
//! This/// use gaiaci_core::lua::register_gaia_lib;module contains the core functionality of GaiaCI, providing essential
//! infrastructure components that support the continuous integration system.
//! 
//! ## Architecture Overview
//! 
//! The core module is organized into two main subsystems:
//! 
//! ### Logging System (`log`)
//! 
//! A flexible, thread-safe logging framework with a sink-based architecture:
//! - **Configurable Outputs** - Route logs to stdout, files, network, or custom destinations
//! - **Multiple Log Levels** - Trace, Debug, Info, Warn, Error with proper ordering
//! - **Thread Safety** - Safe concurrent logging across multiple threads
//! - **Global Interface** - Convenient global functions for easy integration
//! 
//! ### Lua Integration (`lua`)
//! 
//! Comprehensive Lua scripting capabilities for CI pipeline automation:
//! - **Standard Library** - Rich set of modules for common CI operations
//! - **File System Operations** - Read, write, copy, and manage files and directories
//! - **Shell Command Execution** - Run system commands with full environment control
//! - **Environment Variables** - Access and manage system environment
//! - **Assertions** - Comprehensive testing and validation functions
//! - **Logging Integration** - Direct access to the logging system from Lua scripts
//! 
//! ## Usage Example
//! 
//! ```rust
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! use gaiaci_core::log::{set_logger, info};
//! use gaiaci_core::log::sinks::stdout::StdoutSink;
//! use gaiaci_core::lua::register_gaia_lib;
//! use mlua::Lua;
//! 
//! // Set up logging
//! set_logger(StdoutSink);
//! info("GaiaCI core initialized");
//! 
//! // Set up Lua environment with GaiaCI modules
//! let lua = Lua::new();
//! register_gaia_lib(&lua)?;
//! 
//! // Now Lua scripts can use: gaia.log.info(), gaia.fs.read(), gaia.shell.run(), etc.
//! # Ok(())
//! # }
//! ```
//! 
//! ## Design Principles
//! 
//! - **Modularity** - Clear separation of concerns between logging and Lua integration
//! - **Thread Safety** - All components designed for safe concurrent access
//! - **Extensibility** - Plugin-based architecture allows for easy expansion
//! - **Performance** - Efficient implementations suitable for CI/CD workloads
//! - **Reliability** - Robust error handling and graceful degradation

pub mod log;
pub mod lua;