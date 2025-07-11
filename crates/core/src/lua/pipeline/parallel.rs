//! # Parallel Pipeline Execution (Compatibility Layer)
//!
//! This module provides backward compatibility while the new modular
//! parallel execution system is being integrated.

// Re-export the new clean implementation
pub use super::parallel::*;

// Legacy compatibility - will be removed in future versions
pub use super::parallel::ParallelExecutor as ParallelPipelineExecutor;
