//! # Logging Module
//! 
//! This module provides a flexible, thread-safe logging system for GaiaCI.
//! It implements a sink-based architecture where log messages can be routed
//! to different outputs (stdout, files, network, etc.) based on configuration.
//! 
//! ## Architecture
//! 
//! The logging system consists of several key components:
//! 
//! - **LogLevel** - Enumeration of log severity levels (Trace, Debug, Info, Warn, Error)
//! - **LogSink** - Trait for implementing custom log output destinations
//! - **Global Logger** - Thread-safe global logging interface with convenience functions
//! - **Built-in Sinks** - Pre-implemented sinks for common output destinations
//! 
//! ## Usage
//! 
//! ```rust
//! use gaiaci::core::log::{set_logger, info, warn, error};
//! use gaiaci::core::log::sinks::stdout::StdoutSink;
//! 
//! // Set up logging
//! set_logger(StdoutSink);
//! 
//! // Use logging functions
//! info("Application starting");
//! warn("Configuration file not found, using defaults");
//! error("Failed to connect to database");
//! ```
//! 
//! ## Thread Safety
//! 
//! The logging system is designed to be thread-safe and can be used from multiple
//! threads simultaneously. The global logger uses RwLock for safe concurrent access.

pub mod levels;
pub mod sink;
pub mod sinks;
pub mod global;
pub mod formatters;

pub use levels::LogLevel;
pub use sink::LogSink;
pub use formatters::{LogFormatter, LogContext, SimpleFormatter, DetailedFormatter, TemplateFormatter};
pub use global::{set_logger, log, info, warn, error, debug, trace};
