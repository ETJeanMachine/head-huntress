//! Integrations with external services and infrastructure.

/// Job-source integrations.
#[path = "job-sources/mod.rs"]
pub mod job_sources;
/// Language-model integrations.
pub mod llm;
/// Document-rendering integrations.
pub mod rendering;
