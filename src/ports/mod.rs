//! Interfaces owned by the application and implemented by external adapters.

/// Semantic job assessment interfaces.
pub mod assessment;
/// Job-source interfaces.
pub mod job_source;
/// Job-storage interfaces.
pub mod job_store;
/// Language-model provider interfaces.
pub mod llm;
/// Resume-rendering interfaces.
pub mod renderer;
