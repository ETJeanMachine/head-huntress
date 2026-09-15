//! Core domain and application logic for Head Huntress.

/// Application use cases that coordinate domain operations.
pub mod application;
/// Current command-line application shell.
pub mod cli;
/// Domain logic for jobs, candidate profiles, filtering, and resume planning.
pub mod logic;
