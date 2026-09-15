//! Job ingestion snapshots and normalized job listings.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// The source-provided identifier for a listing.
///
/// This is intentionally kept separate from the internal UUID stored on
/// [`Job`]. A source identifier can be missing, unstable, or reused by the
/// source.
pub type SourceJobId = String;

/// A job listing exactly as received from a source adapter.
///
/// Raw jobs should be persisted before normalization. Keeping the original
/// payload makes parser improvements and failed ingestion runs replayable.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RawJob {
    /// The identifier supplied by the source, if one exists.
    pub id: Option<SourceJobId>,
    /// The stable identifier of the source adapter that produced this listing.
    pub source: String,
    /// The URL from which the raw listing was fetched.
    pub url: String,
    /// The time at which this raw listing was fetched.
    pub fetched_at: DateTime<Utc>,
    /// The unmodified content returned by the source.
    pub content: String,
}

impl RawJob {
    /// Creates a raw job snapshot with the current time as its fetch time.
    pub fn new(
        id: Option<SourceJobId>,
        source: impl Into<String>,
        url: impl Into<String>,
        content: impl Into<String>,
    ) -> Self {
        Self {
            id,
            source: source.into(),
            url: url.into(),
            fetched_at: Utc::now(),
            content: content.into(),
        }
    }
}

/// The canonical, normalized representation of a job listing.
#[derive(Debug, Clone, PartialEq)]
pub struct Job {
    /// Internal UUIDv7 identifier. This is not the identifier from a job board.
    pub id: String,
    /// The identifier of the source adapter that supplied this job.
    pub source: String,
    /// The source-provided identifier, when available.
    pub source_job_id: Option<SourceJobId>,
    /// The canonical URL for the listing.
    pub url: String,
    /// The normalized title of the position.
    pub title: String,
    /// The normalized name of the hiring company.
    pub company: String,
    /// The normalized text of the job description.
    pub description: String,
    /// The location or locations associated with the position.
    pub location: Option<String>,
    /// The work-location policy, when it could be determined.
    pub remote_policy: Option<RemotePolicy>,
    /// The employment arrangement, when it could be determined.
    pub employment_type: Option<EmploymentType>,
    /// The advertised compensation, when it was provided and parsed.
    pub compensation: Option<Compensation>,
    /// The date on which the listing was originally posted, when known.
    pub posted_at: Option<DateTime<Utc>>,
    /// The first time this canonical job was observed by the application.
    pub first_seen_at: DateTime<Utc>,
    /// The most recent time this canonical job was observed by the application.
    pub last_seen_at: DateTime<Utc>,
}

impl Job {
    /// Creates a canonical job from normalized fields.
    ///
    /// Fields that cannot be reliably extracted are represented as `None`
    /// rather than being guessed. This is important for later rule evaluation.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        source: impl Into<String>,
        source_job_id: Option<SourceJobId>,
        url: impl Into<String>,
        title: impl Into<String>,
        company: impl Into<String>,
        description: impl Into<String>,
        location: Option<String>,
        remote_policy: Option<RemotePolicy>,
        employment_type: Option<EmploymentType>,
        compensation: Option<Compensation>,
        posted_at: Option<DateTime<Utc>>,
    ) -> Self {
        let now = Utc::now();

        Self {
            id: Uuid::now_v7().to_string(),
            source: source.into(),
            source_job_id,
            url: url.into(),
            title: title.into(),
            company: company.into(),
            description: description.into(),
            location,
            remote_policy,
            employment_type,
            compensation,
            posted_at,
            first_seen_at: now,
            last_seen_at: now,
        }
    }

    /// Records that the source still contains this listing.
    pub fn mark_seen(&mut self, seen_at: DateTime<Utc>) {
        if seen_at > self.last_seen_at {
            self.last_seen_at = seen_at;
        }
    }

    /// Returns the source identifier used for matching listings from the same
    /// source. URL matching should be used as a fallback when this is `None`.
    pub fn source_key(&self) -> Option<(&str, &str)> {
        self.source_job_id
            .as_deref()
            .map(|source_job_id| (self.source.as_str(), source_job_id))
    }
}

/// Describes where the work is expected to be performed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RemotePolicy {
    /// The position is expected to be performed at a workplace.
    OnSite,
    /// The position combines remote work and work at a workplace.
    Hybrid,
    /// The position is intended to be performed remotely.
    Remote,
}

/// Describes the employment arrangement for a job.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum EmploymentType {
    /// A standard full-time position.
    FullTime,
    /// A part-time position.
    PartTime,
    /// A position engaged under a contract.
    Contract,
    /// A temporary position.
    Temporary,
    /// An internship or other explicitly educational position.
    Internship,
    /// An employment arrangement not represented by the other variants.
    Other,
}

/// Represents the compensation advertised for a job.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct Compensation {
    /// The lower end of the advertised range, if provided.
    pub minimum: Option<u64>,
    /// The upper end of the advertised range, if provided.
    pub maximum: Option<u64>,
    /// The ISO 4217 or source-provided currency code.
    pub currency: String,
    /// The period over which the compensation is stated.
    pub period: CompensationPeriod,
}

impl Compensation {
    /// Creates a compensation range with an optional minimum and maximum.
    pub fn new(
        minimum: Option<u64>,
        maximum: Option<u64>,
        currency: impl Into<String>,
        period: CompensationPeriod,
    ) -> Self {
        Self {
            minimum,
            maximum,
            currency: currency.into(),
            period,
        }
    }
}

/// Describes the period associated with a compensation amount.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CompensationPeriod {
    /// Compensation is stated per hour.
    Hourly,
    /// Compensation is stated per month.
    Monthly,
    /// Compensation is stated per year.
    Annual,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn raw_job_uses_the_fetch_time_by_default() {
        let before = Utc::now();
        let job = RawJob::new(
            Some("source-1".to_owned()),
            "example",
            "https://example.test/job",
            "content",
        );
        let after = Utc::now();

        assert!(job.fetched_at >= before);
        assert!(job.fetched_at <= after);
    }

    #[test]
    fn mark_seen_does_not_move_a_job_backwards() {
        let mut job = Job::new(
            "example",
            Some("source-1".to_owned()),
            "https://example.test/job",
            "Software Engineer",
            "Example Co",
            "Build useful things.",
            None,
            None,
            None,
            None,
            None,
        );
        let original_last_seen = job.last_seen_at;

        job.mark_seen(original_last_seen - chrono::TimeDelta::seconds(1));
        assert_eq!(job.last_seen_at, original_last_seen);

        let later = original_last_seen + chrono::TimeDelta::seconds(1);
        job.mark_seen(later);
        assert_eq!(job.last_seen_at, later);
    }
}
