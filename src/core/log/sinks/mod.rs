//! # Log Sinks
//! 
//! This module contains built-in implementations of the `LogSink` trait
//! for common log output destinations. These sinks provide ready-to-use
//! logging capabilities for various scenarios.
//! 
//! ## Available Sinks
//! 
//! - **stdout** - Outputs log messages to standard output with basic formatting
//! 
//! ## Future Sinks
//! 
//! Additional sinks that could be implemented:
//! - **file** - Write logs to rotating files
//! - **network** - Send logs to remote logging services
//! - **syslog** - Integration with system logging
//! - **buffer** - Buffered output for high-throughput scenarios

pub mod stdout;