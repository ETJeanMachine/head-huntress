//! Interfaces for rendering validated resume plans.

use crate::logic::profile::CandidateProfile;
use crate::logic::resume::ResumePlan;
use std::error::Error;
use std::fmt;
use std::path::Path;

/// Application-owned interface for resume renderers.
pub trait ResumeRenderer: Send + Sync {
    /// Renders a validated resume plan and candidate profile into a document.
    fn render(
        &self,
        plan: &ResumePlan,
        profile: &CandidateProfile,
    ) -> Result<String, RendererError>;
}

/// Errors returned while validating or rendering a resume.
#[derive(Debug)]
pub enum RendererError {
    /// The resume plan references invalid or unverified profile evidence.
    InvalidPlan(String),
    /// The template could not be parsed or rendered.
    Template(String),
    /// A rendered document could not be read or written.
    Io(String),
}

impl fmt::Display for RendererError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidPlan(message) => write!(formatter, "invalid resume plan: {message}"),
            Self::Template(message) => write!(formatter, "template rendering error: {message}"),
            Self::Io(message) => write!(formatter, "renderer I/O error: {message}"),
        }
    }
}

impl Error for RendererError {}

/// Reads a renderer template from a filesystem path.
pub fn read_template(path: impl AsRef<Path>) -> Result<String, RendererError> {
    std::fs::read_to_string(path).map_err(|error| RendererError::Io(error.to_string()))
}
