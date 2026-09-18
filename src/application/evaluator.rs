//! Application service for evaluating jobs against configured rules.

use crate::logic::job::Job;
use crate::logic::rules::{FilterResult, JobRules, SemanticAssessment, searchable_job_text};
use crate::ports::assessment::{AssessmentError, SemanticAssessor};

/// Coordinates deterministic and semantic evaluation for jobs.
#[derive(Debug, Clone)]
pub struct JobEvaluator {
    rules: JobRules,
}

impl JobEvaluator {
    /// Creates an evaluator using the supplied rule set.
    pub fn new(rules: JobRules) -> Self {
        Self { rules }
    }

    /// Returns the rules used by this evaluator.
    pub fn rules(&self) -> &JobRules {
        &self.rules
    }

    /// Evaluates one job and attaches its internal ID to the result.
    pub fn evaluate(&self, job: &Job, assessments: &[SemanticAssessment]) -> JobEvaluation {
        JobEvaluation {
            job_id: job.id.clone(),
            result: self.rules.evaluate(job, assessments),
        }
    }

    /// Requests semantic assessments and evaluates the job.
    pub async fn evaluate_with_provider<P: SemanticAssessor + ?Sized>(
        &self,
        provider: &P,
        job: &Job,
    ) -> Result<JobEvaluation, AssessmentError> {
        let state = searchable_job_text(job);
        let assessments = provider.assess_job(&state, &self.rules).await?;
        Ok(self.evaluate(job, &assessments))
    }
}

/// Evaluation result associated with a specific job.
#[derive(Debug, Clone, PartialEq)]
pub struct JobEvaluation {
    /// Internal ID of the evaluated job.
    pub job_id: String,
    /// The auditable rule evaluation result.
    pub result: FilterResult,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::logic::job::Job;
    use crate::ports::assessment::AssessmentFuture;

    struct MockAssessor;

    impl SemanticAssessor for MockAssessor {
        fn assess_job<'a>(
            &'a self,
            _state: &'a str,
            _rules: &'a JobRules,
        ) -> AssessmentFuture<'a, Vec<SemanticAssessment>> {
            Box::pin(async { Ok(Vec::new()) })
        }
    }

    #[test]
    fn evaluator_keeps_the_job_identity() {
        let job = Job::new(
            "example",
            None,
            "https://example.test/job",
            "Engineer",
            "Example Co",
            "Build useful things.",
            None,
            None,
            None,
            None,
            None,
        );
        let evaluator = JobEvaluator::new(JobRules::default());

        let evaluation = evaluator.evaluate(&job, &[]);

        assert_eq!(evaluation.job_id, job.id);
    }

    #[tokio::test]
    async fn evaluator_can_use_an_llm_provider_port() {
        let job = Job::new(
            "example",
            None,
            "https://example.test/job",
            "Engineer",
            "Example Co",
            "Build useful things.",
            None,
            None,
            None,
            None,
            None,
        );
        let evaluator = JobEvaluator::new(JobRules::default());

        let evaluation = evaluator
            .evaluate_with_provider(&MockAssessor, &job)
            .await
            .expect("mock assessor succeeds");

        assert_eq!(evaluation.job_id, job.id);
    }
}
