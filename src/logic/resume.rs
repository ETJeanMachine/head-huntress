//! Evidence-backed resume planning.

use super::profile::CandidateProfile;

/// A structured plan for tailoring a resume to a particular job.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResumePlan {
    /// Internal ID of the target job.
    pub job_id: String,
    /// Tailored summary proposed for the resume.
    pub summary: Option<String>,
    /// Experience IDs selected for inclusion.
    pub selected_experience_ids: Vec<String>,
    /// Skill names selected for inclusion.
    pub selected_skills: Vec<String>,
    /// Rewritten bullets that retain links to source achievements.
    pub bullet_rewrites: Vec<BulletRewrite>,
    /// Warnings that should be shown during human review.
    pub warnings: Vec<String>,
}

impl ResumePlan {
    /// Creates an empty plan for a target job.
    pub fn new(job_id: impl Into<String>) -> Self {
        Self {
            job_id: job_id.into(),
            summary: None,
            selected_experience_ids: Vec::new(),
            selected_skills: Vec::new(),
            bullet_rewrites: Vec::new(),
            warnings: Vec::new(),
        }
    }

    /// Validates that every selected source item exists in the candidate profile.
    ///
    /// This prevents a renderer from producing a resume containing unsupported
    /// or fabricated experience references.
    pub fn validate(&self, profile: &CandidateProfile) -> Result<(), Vec<ResumeValidationError>> {
        let mut errors = Vec::new();

        for experience_id in &self.selected_experience_ids {
            if !profile.has_experience(experience_id) {
                errors.push(ResumeValidationError::MissingExperience {
                    experience_id: experience_id.clone(),
                });
            }
        }

        for rewrite in &self.bullet_rewrites {
            match profile.is_achievement_verified(&rewrite.source_achievement_id) {
                None => errors.push(ResumeValidationError::MissingAchievement {
                    achievement_id: rewrite.source_achievement_id.clone(),
                }),
                Some(false) => errors.push(ResumeValidationError::UnverifiedAchievement {
                    achievement_id: rewrite.source_achievement_id.clone(),
                }),
                Some(true) => {}
            }
            if rewrite.text.trim().is_empty() {
                errors.push(ResumeValidationError::EmptyBullet {
                    achievement_id: rewrite.source_achievement_id.clone(),
                });
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

/// A generated bullet tied to a verified source achievement.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BulletRewrite {
    /// ID of the original achievement in the candidate profile.
    pub source_achievement_id: String,
    /// Tailored text intended for the rendered resume.
    pub text: String,
}

impl BulletRewrite {
    /// Creates a rewrite linked to a source achievement.
    pub fn new(source_achievement_id: impl Into<String>, text: impl Into<String>) -> Self {
        Self {
            source_achievement_id: source_achievement_id.into(),
            text: text.into(),
        }
    }
}

/// A problem that prevents a resume plan from being safely rendered.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResumeValidationError {
    /// The plan references an experience absent from the profile.
    MissingExperience {
        /// The missing experience ID.
        experience_id: String,
    },
    /// The plan references an achievement absent from the profile.
    MissingAchievement {
        /// The missing achievement ID.
        achievement_id: String,
    },
    /// The plan references an achievement that has not been verified.
    UnverifiedAchievement {
        /// The unverified achievement ID.
        achievement_id: String,
    },
    /// A rewrite has no usable text.
    EmptyBullet {
        /// The achievement associated with the empty rewrite.
        achievement_id: String,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::logic::profile::{Achievement, Experience};

    #[test]
    fn plan_validation_accepts_profile_evidence() {
        let mut profile = CandidateProfile::new("Jane Doe");
        let mut experience = Experience::new("experience-1", "Example Co", "Engineer");
        experience
            .achievements
            .push(Achievement::new("achievement-1", "Improved reliability."));
        profile.experiences.push(experience);

        let mut plan = ResumePlan::new("job-1");
        plan.selected_experience_ids.push("experience-1".to_owned());
        plan.bullet_rewrites.push(BulletRewrite::new(
            "achievement-1",
            "Improved reliability by 40%.",
        ));

        assert!(plan.validate(&profile).is_ok());
    }

    #[test]
    fn plan_validation_rejects_unverified_evidence() {
        let mut profile = CandidateProfile::new("Jane Doe");
        let mut experience = Experience::new("experience-1", "Example Co", "Engineer");
        let mut achievement = Achievement::new("achievement-1", "Unverified claim.");
        achievement.verified = false;
        experience.achievements.push(achievement);
        profile.experiences.push(experience);

        let mut plan = ResumePlan::new("job-1");
        plan.bullet_rewrites
            .push(BulletRewrite::new("achievement-1", "Unverified claim."));

        let errors = plan.validate(&profile).expect_err("plan should be invalid");

        assert!(matches!(
            errors[0],
            ResumeValidationError::UnverifiedAchievement { .. }
        ));
    }

    #[test]
    fn plan_validation_rejects_unknown_evidence() {
        let profile = CandidateProfile::new("Jane Doe");
        let mut plan = ResumePlan::new("job-1");
        plan.bullet_rewrites
            .push(BulletRewrite::new("missing", "Unsupported claim."));

        let errors = plan.validate(&profile).expect_err("plan should be invalid");

        assert_eq!(errors.len(), 1);
        assert!(matches!(
            errors[0],
            ResumeValidationError::MissingAchievement { .. }
        ));
    }
}
