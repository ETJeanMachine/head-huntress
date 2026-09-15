//! Candidate profile data used for job evaluation and resume planning.

/// A candidate's reusable source of truth for job matching and resume generation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CandidateProfile {
    /// The candidate's name.
    pub name: String,
    /// A general-purpose professional summary, when available.
    pub summary: Option<String>,
    /// The candidate's professional experience.
    pub experiences: Vec<Experience>,
    /// Skills that may be selected for a job-specific resume.
    pub skills: Vec<String>,
}

impl CandidateProfile {
    /// Creates an empty profile for the supplied candidate name.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            summary: None,
            experiences: Vec::new(),
            skills: Vec::new(),
        }
    }

    /// Returns whether the profile contains an experience with the given ID.
    pub fn has_experience(&self, experience_id: &str) -> bool {
        self.experiences
            .iter()
            .any(|experience| experience.id == experience_id)
    }

    /// Returns whether the profile contains an achievement with the given ID.
    pub fn has_achievement(&self, achievement_id: &str) -> bool {
        self.experiences.iter().any(|experience| {
            experience
                .achievements
                .iter()
                .any(|achievement| achievement.id == achievement_id)
        })
    }

    /// Returns whether an achievement exists and is marked as verified.
    pub fn is_achievement_verified(&self, achievement_id: &str) -> Option<bool> {
        self.experiences
            .iter()
            .flat_map(|experience| &experience.achievements)
            .find(|achievement| achievement.id == achievement_id)
            .map(|achievement| achievement.verified)
    }
}

/// A position held by the candidate.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Experience {
    /// Stable identifier used to reference this experience from a resume plan.
    pub id: String,
    /// The employer or organization.
    pub company: String,
    /// The title held by the candidate.
    pub title: String,
    /// The beginning of the experience, represented in the profile's chosen format.
    pub start_date: Option<String>,
    /// The end of the experience, or `None` for a current position.
    pub end_date: Option<String>,
    /// Verifiable accomplishments from this experience.
    pub achievements: Vec<Achievement>,
}

impl Experience {
    /// Creates an experience with no achievements or dates.
    pub fn new(
        id: impl Into<String>,
        company: impl Into<String>,
        title: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            company: company.into(),
            title: title.into(),
            start_date: None,
            end_date: None,
            achievements: Vec::new(),
        }
    }
}

/// A verifiable accomplishment that may be used in a tailored resume.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Achievement {
    /// Stable identifier used to preserve provenance in generated documents.
    pub id: String,
    /// The source text describing what the candidate did.
    pub text: String,
    /// Skills or concepts demonstrated by the accomplishment.
    pub tags: Vec<String>,
    /// Whether the candidate has verified this claim.
    pub verified: bool,
}

impl Achievement {
    /// Creates a verified achievement from source text.
    pub fn new(id: impl Into<String>, text: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            text: text.into(),
            tags: Vec::new(),
            verified: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn profile_can_find_nested_achievements() {
        let mut profile = CandidateProfile::new("Jane Doe");
        let mut experience = Experience::new("experience-1", "Example Co", "Engineer");
        experience
            .achievements
            .push(Achievement::new("achievement-1", "Improved reliability."));
        profile.experiences.push(experience);

        assert!(profile.has_experience("experience-1"));
        assert!(profile.has_achievement("achievement-1"));
        assert!(!profile.has_achievement("missing"));
    }
}
