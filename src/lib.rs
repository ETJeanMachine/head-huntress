//! Core domain and application logic for Head Huntress.

/// Integrations with external services and infrastructure.
pub mod adapters;
/// Application use cases that coordinate domain operations.
pub mod application;
/// Current command-line application shell.
pub mod cli;
/// Domain logic for jobs, candidate profiles, filtering, and resume planning.
pub mod logic;
/// Interfaces implemented by external adapters.
pub mod ports;
