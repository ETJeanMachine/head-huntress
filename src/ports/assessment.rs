//! Interfaces for semantic job assessment providers.

use crate::logic::rules::{JobRules, SemanticAssessment};
use std::error::Error;
use std::fmt;
use std::future::Future;
use std::pin::Pin;

/// A boxed future returned by a [`SemanticAssessor`].
pub type AssessmentFuture<'a, T> =
    Pin<Box<dyn Future<Output = Result<T, AssessmentError>> + Send + 'a>>;

/// Application-owned interface for semantic job assessment.
///
/// Implementations evaluate a job's state text against a configured rule set
/// and return calibrated, auditable assessments. TypeSafe's Jev is the default
/// provider; generative LLMs may implement this port as an alternative.
pub trait SemanticAssessor: Send + Sync {
    /// Assesses the job state text against the supplied rules.
    fn assess_job<'a>(
        &'a self,
        state: &'a str,
        rules: &'a JobRules,
    ) -> AssessmentFuture<'a, Vec<SemanticAssessment>>;
}

/// Errors that can occur while obtaining or decoding semantic assessments.
#[derive(Debug)]
pub enum AssessmentError {
    /// The assessor could not be configured from the supplied settings.
    Configuration(String),
    /// The assessor request failed before a usable response was received.
    Transport(String),
    /// The assessor returned content that could not be decoded as assessments.
    InvalidResponse(String),
    /// The assessor returned an application-level error.
    Provider(String),
}

impl fmt::Display for AssessmentError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Configuration(message) => {
                write!(formatter, "assessment configuration error: {message}")
            }
            Self::Transport(message) => write!(formatter, "assessment transport error: {message}"),
            Self::InvalidResponse(message) => {
                write!(formatter, "invalid assessment response: {message}")
            }
            Self::Provider(message) => write!(formatter, "assessment provider error: {message}"),
        }
    }
}

impl Error for AssessmentError {}
