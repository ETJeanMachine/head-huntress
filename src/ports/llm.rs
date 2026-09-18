//! Interfaces for language-model providers.

use crate::logic::resume::ResumePlan;
use std::error::Error;
use std::fmt;
use std::future::Future;
use std::pin::Pin;

/// A boxed future returned by an [`LlmProvider`].
pub type LlmFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

/// Provider-independent interface for language-model text generation.
///
/// Semantic job assessment is intentionally not part of this port; it is
/// owned by [`crate::ports::assessment::SemanticAssessor`], which is
/// implemented by judgment providers such as TypeSafe's Jev by default.
pub trait LlmProvider: Send + Sync {
    /// Generates a structured, evidence-backed resume plan.
    fn generate_resume_plan<'a>(
        &'a self,
        prompt: &'a str,
    ) -> LlmFuture<'a, Result<ResumePlan, LlmError>>;
}

/// Errors that can occur while obtaining or decoding an LLM response.
#[derive(Debug)]
pub enum LlmError {
    /// The provider could not be configured from the supplied settings.
    Configuration(String),
    /// The provider request failed before a usable response was received.
    Transport(String),
    /// The provider returned content that could not be decoded as a plan.
    InvalidResponse(String),
    /// The provider returned an application-level error.
    Provider(String),
}

impl fmt::Display for LlmError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Configuration(message) => write!(formatter, "LLM configuration error: {message}"),
            Self::Transport(message) => write!(formatter, "LLM transport error: {message}"),
            Self::InvalidResponse(message) => write!(formatter, "Invalid LLM response: {message}"),
            Self::Provider(message) => write!(formatter, "LLM provider error: {message}"),
        }
    }
}

impl Error for LlmError {}
