//! Domain logic for job ingestion, filtering, and resume planning.

/// Job listings and normalized job metadata.
pub mod job;
/// Candidate experience and evidence used for tailoring.
pub mod profile;
/// Evidence-backed resume plans.
pub mod resume;
/// Deterministic and semantic job filtering.
pub mod rules;
