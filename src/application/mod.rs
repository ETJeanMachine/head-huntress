//! Application use cases that coordinate the domain logic.

/// Coordinates rule-based and semantic job evaluation.
pub mod evaluator;
/// Creates conservative, evidence-backed resume plans.
pub mod generator;
/// Coordinates ingestion and deduplication of normalized jobs.
pub mod ingester;
/// Tracks human review decisions for evaluated jobs.
pub mod reviewer;
