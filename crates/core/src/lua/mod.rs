//! # Lua Integration Module
//! 
//! This module provides comprehensive Lua scripting capabilities for GaiaCI,
//! enabling powerful and flexible automation of continuous integration pipelines.
//! The module integrates the mlua library with custom standard library extensions
//! specifically designed for CI/CD workflows.
//! 
//! ## Architecture
//! 
//! The Lua integration is built around a standard library that provides
//! CI-focused modules accessible from Lua scripts through the global `gaia` namespace.
//! 
//! ### Standard Library Modules
//! 
//! The standard library contains specialized modules for common CI operations:
//! 
//! - **`gaia.log`** - Structured logging with multiple severity levels
//! - **`gaia.shell`** - Execute system commands with environment control
//! - **`gaia.fs`** - File system operations (read, write, copy, list, etc.)
//! - **`gaia.env`** - Environment variable access and management
//! - **`gaia.assert`** - Comprehensive assertion functions for testing
//! 
//! ## Integration Pattern
//! 
//! All standard library modules implement the `GaiaModule` trait, providing
//! a consistent interface for registration and initialization within the
//! Lua environment.
//! 
//! ## Usage Example
//! 
//! ```rust
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! use mlua::Lua;
//! use gaiaci_core::lua::lib::register_gaia_lib;
//! 
//! // Create Lua environment
//! let lua = Lua::new();
//! 
//! // Register GaiaCI standard library
//! register_gaia_lib(&lua)?;
//! 
//! // Execute Lua script with GaiaCI capabilities
//! lua.load(r#"
//!     -- Log pipeline start
//!     gaia.log.info("Starting CI pipeline")
//!     
//!     -- Check if build directory exists
//!     if gaia.fs.exists("build") then
//!         gaia.log.info("Build directory found")
//!         
//!         -- List build artifacts
//!         local files = gaia.fs.list_dir("build")
//!         for i, file in ipairs(files) do
//!             gaia.log.info("Artifact: " .. file)
//!         end
//!     else
//!         gaia.log.warn("Build directory not found, creating...")
//!         gaia.shell.run({cmd = "mkdir -p build"})
//!     end
//!     
//!     -- Run tests
//!     local result = gaia.shell.run({
//!         cmd = "npm test",
//!         capture = true,
//!         cwd = ".",
//!         env = {NODE_ENV = "test"}
//!     })
//!     
//!     if result.success then
//!         gaia.log.info("Tests passed!")
//!     else
//!         gaia.log.error("Tests failed: " .. result.stderr)
//!     end
//! "#).exec()?;
//! # Ok(())
//! # }
//! ```
//! 
//! ## Design Benefits
//! 
//! - **Declarative Pipelines** - Write CI logic in clear, readable Lua scripts
//! - **Dynamic Behavior** - Full programming language capabilities for complex logic
//! - **Error Handling** - Lua's error handling integrates with Rust's Result types
//! - **Sandboxing** - Controlled execution environment for security
//! - **Extensibility** - Easy to add new modules and functionality
//! 
//! ## Security Considerations
//! 
//! The Lua environment provides powerful system access through shell commands
//! and file operations. In production deployments, consider:
//! 
//! - Input validation for user-provided scripts
//! - Sandboxing mechanisms to limit system access
//! - Resource limits to prevent infinite loops or excessive memory usage
//! - Audit logging of executed operations

// Common types and traits
pub mod common;

// Standard library registration
pub mod lib;

// Lua runtime for pipeline execution
pub mod runtime;

// Standard library modules
pub mod log;
pub mod shell;
pub mod fs;
pub mod env;
pub mod assert;
pub mod pipeline;

// Re-export commonly used items for convenience
pub use common::GaiaModule;
pub use lib::register_gaia_lib;
pub use runtime::LuaRuntime;
pub use pipeline::{Pipeline, PipelineStep, PipelineExecutor, PipelineParser};
