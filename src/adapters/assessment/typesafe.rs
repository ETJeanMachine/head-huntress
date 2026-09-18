//! TypeSafe Jev implementation of the semantic-assessment port.

use crate::logic::rules::{AssessmentOutcome, JobRules, SemanticAssessment, SemanticRule};
use crate::ports::assessment::{AssessmentError, AssessmentFuture, SemanticAssessor};
use reqwest::header::{HeaderMap, RETRY_AFTER};
use reqwest::{Client, StatusCode};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::time::Duration;

/// The TypeSafe evaluation endpoint used for semantic assessments.
const DEFAULT_BASE_URL: &str = "https://api.typesafe.ai/v1/systemone";
/// TypeSafe's flagship judgment model.
const DEFAULT_MODEL: &str = "jev-latest";
/// The maximum number of attempts made per evaluation request.
const MAX_ATTEMPTS: u32 = 3;
/// The initial backoff applied to throttled or failed requests.
const INITIAL_BACKOFF: Duration = Duration::from_millis(500);

/// A TypeSafe Jev-backed [`SemanticAssessor`].
///
/// Each semantic rule becomes one batched Noul question, and the returned
/// yes-probability becomes the assessment confidence. Jev returns calibrated
/// judgments rather than generated text, so assessments carry no evidence
/// quotes by construction. The rule set's free-form `semantic_prompt`
/// preference is a generative-provider concern and is not sent to Jev.
#[derive(Debug, Clone)]
pub struct TypeSafeAssessor {
    client: Client,
    base_url: String,
    model: String,
    api_key: String,
}

impl TypeSafeAssessor {
    /// Creates an assessor with an API key and a TypeSafe model identifier.
    pub fn new(
        api_key: impl Into<String>,
        model: impl Into<String>,
    ) -> Result<Self, AssessmentError> {
        let client = Client::builder()
            .user_agent("head-huntress/0.1")
            .timeout(Duration::from_secs(30))
            .build()
            .map_err(|error| AssessmentError::Configuration(error.to_string()))?;

        Ok(Self {
            client,
            base_url: DEFAULT_BASE_URL.to_owned(),
            model: model.into(),
            api_key: api_key.into(),
        })
    }

    /// Creates an assessor using `TYPESAFE_API_KEY` and the default model.
    pub fn from_env() -> Result<Self, AssessmentError> {
        let api_key = std::env::var("TYPESAFE_API_KEY").map_err(|error| {
            AssessmentError::Configuration(format!("TYPESAFE_API_KEY is unavailable: {error}"))
        })?;
        Self::new(api_key, DEFAULT_MODEL)
    }

    /// Overrides the evaluation endpoint, which keeps tests off the network.
    pub fn with_base_url(mut self, base_url: impl Into<String>) -> Self {
        self.base_url = base_url.into();
        self
    }

    async fn evaluate(
        &self,
        request: &EvaluationRequest<'_>,
    ) -> Result<EvaluationResponse, AssessmentError> {
        let mut backoff = INITIAL_BACKOFF;
        let mut attempt = 1;

        loop {
            let response = self
                .client
                .post(&self.base_url)
                .bearer_auth(&self.api_key)
                .json(request)
                .send()
                .await
                .map_err(|error| AssessmentError::Transport(error.to_string()))?;
            let status = response.status();

            if attempt < MAX_ATTEMPTS
                && (status == StatusCode::TOO_MANY_REQUESTS || status.is_server_error())
            {
                attempt += 1;
                let delay = retry_delay(response.headers()).unwrap_or(backoff);
                tokio::time::sleep(delay).await;
                backoff *= 2;
                continue;
            }

            let body = response
                .text()
                .await
                .map_err(|error| AssessmentError::Transport(error.to_string()))?;

            if !status.is_success() {
                return Err(AssessmentError::Provider(format!(
                    "typesafe returned {status}: {body}"
                )));
            }

            return serde_json::from_str(&body).map_err(|error| {
                AssessmentError::InvalidResponse(format!(
                    "could not decode typesafe response: {error}"
                ))
            });
        }
    }
}

impl SemanticAssessor for TypeSafeAssessor {
    fn assess_job<'a>(
        &'a self,
        state: &'a str,
        rules: &'a JobRules,
    ) -> AssessmentFuture<'a, Vec<SemanticAssessment>> {
        Box::pin(async move {
            if rules.semantic_rules.is_empty() {
                return Ok(Vec::new());
            }

            let request = build_request(state, rules, &self.model);
            let response = self.evaluate(&request).await?;
            Ok(map_answers(&response, &rules.semantic_rules))
        })
    }
}

/// The wire shape of a TypeSafe evaluation request.
#[derive(Debug, Serialize)]
struct EvaluationRequest<'a> {
    state: &'a str,
    model: &'a str,
    questions: BTreeMap<&'a str, NoulQuestion>,
}

/// A single Noul (yes/no) question for one semantic rule.
#[derive(Debug, Serialize)]
struct NoulQuestion {
    #[serde(rename = "type")]
    kind: &'static str,
    instructions: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    criteria: Option<NoulCriteria>,
}

/// Descriptions of what a yes and a no mean for a Noul question.
#[derive(Debug, Serialize)]
struct NoulCriteria {
    #[serde(rename = "true")]
    when_yes: String,
    #[serde(rename = "false")]
    when_no: String,
}

/// The wire shape of a TypeSafe evaluation response.
#[derive(Debug, Deserialize)]
struct EvaluationResponse {
    #[serde(default)]
    answers: BTreeMap<String, Answer>,
}

/// One typed answer, tagged by its question type.
///
/// Choice and Score are parsed today so future rule styles can consume them
/// without a wire-format change, but only Noul answers are read.
#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
#[allow(dead_code)]
enum Answer {
    /// A yes/no probability.
    Noul {
        /// The probability that the answer is yes.
        noul: f64,
    },
    /// A selected option with its distribution confidence.
    Choice {
        /// The highest-probability option.
        choice: String,
        /// Certainty derived from the option distribution.
        confidence: Option<f64>,
    },
    /// A weighted rubric score with its confidence.
    Score {
        /// The probability-weighted score value.
        score: f64,
        /// Certainty derived from the level distribution.
        confidence: Option<f64>,
    },
}

/// Builds one batched Noul question per semantic rule.
fn build_request<'a>(state: &'a str, rules: &'a JobRules, model: &'a str) -> EvaluationRequest<'a> {
    let questions = rules
        .semantic_rules
        .iter()
        .map(|rule| {
            let question = NoulQuestion {
                kind: "noul",
                instructions: format!("Does the job satisfy this preference? {}", rule.description),
                criteria: Some(NoulCriteria {
                    when_yes: rule.description.clone(),
                    when_no: "The job does not satisfy this preference.".to_owned(),
                }),
            };
            (rule.id.as_str(), question)
        })
        .collect();

    EvaluationRequest {
        state,
        model,
        questions,
    }
}

/// Converts TypeSafe answers into calibrated domain assessments.
fn map_answers(response: &EvaluationResponse, rules: &[SemanticRule]) -> Vec<SemanticAssessment> {
    rules
        .iter()
        .map(|rule| match response.answers.get(&rule.id) {
            Some(Answer::Noul { noul }) => {
                let probability = noul.clamp(0.0, 1.0) as f32;
                let outcome = if probability >= rule.minimum_confidence {
                    AssessmentOutcome::Match
                } else if probability <= 1.0 - rule.minimum_confidence {
                    AssessmentOutcome::NoMatch
                } else {
                    AssessmentOutcome::Uncertain
                };
                SemanticAssessment {
                    rule_id: rule.id.clone(),
                    outcome,
                    confidence: probability,
                    evidence: Vec::new(),
                    explanation: format!(
                        "Jev assigned a {probability:.2} probability that this job satisfies: {}",
                        rule.description
                    ),
                }
            }
            answer => SemanticAssessment {
                rule_id: rule.id.clone(),
                outcome: AssessmentOutcome::Uncertain,
                confidence: 0.0,
                evidence: Vec::new(),
                explanation: format!(
                    "TypeSafe returned no usable Noul answer for this rule ({answer:?})."
                ),
            },
        })
        .collect()
}

/// Returns the server-advised retry delay, when a valid header is present.
fn retry_delay(headers: &HeaderMap) -> Option<Duration> {
    headers
        .get(RETRY_AFTER)?
        .to_str()
        .ok()?
        .parse::<u64>()
        .ok()
        .map(Duration::from_secs)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rules() -> JobRules {
        JobRules {
            semantic_rules: vec![SemanticRule::new(
                "backend",
                "Backend or platform engineering focus.",
                1.0,
                0.7,
            )],
            ..JobRules::default()
        }
    }

    #[test]
    fn request_batches_one_noul_question_per_rule() {
        let rules = rules();
        let request = build_request("Backend Engineer at Example Co.", &rules, "jev-latest");

        assert_eq!(request.model, "jev-latest");
        let question = request
            .questions
            .get("backend")
            .expect("rule becomes a question");
        assert_eq!(question.kind, "noul");
        assert!(question.instructions.contains("Backend or platform"));
    }

    #[test]
    fn serialized_request_matches_the_typesafe_shape() {
        let rules = rules();
        let request = build_request("Backend Engineer at Example Co.", &rules, "jev-latest");
        let json = serde_json::to_value(&request).expect("request serializes");

        assert_eq!(json["model"], "jev-latest");
        assert_eq!(json["questions"]["backend"]["type"], "noul");
        assert_eq!(
            json["questions"]["backend"]["criteria"]["true"],
            "Backend or platform engineering focus."
        );
    }

    #[test]
    fn probabilities_map_to_calibrated_outcomes() {
        let rules = rules();
        let response = |noul: f64| EvaluationResponse {
            answers: BTreeMap::from([("backend".to_owned(), Answer::Noul { noul })]),
        };

        let confident = map_answers(&response(0.9), &rules.semantic_rules);
        assert_eq!(confident[0].outcome, AssessmentOutcome::Match);
        assert_eq!(confident[0].confidence, 0.9);

        let refuted = map_answers(&response(0.2), &rules.semantic_rules);
        assert_eq!(refuted[0].outcome, AssessmentOutcome::NoMatch);

        let uncertain = map_answers(&response(0.5), &rules.semantic_rules);
        assert_eq!(uncertain[0].outcome, AssessmentOutcome::Uncertain);
    }

    #[test]
    fn missing_answers_become_uncertain() {
        let rules = rules();
        let response = EvaluationResponse {
            answers: BTreeMap::new(),
        };

        let assessments = map_answers(&response, &rules.semantic_rules);

        assert_eq!(assessments[0].outcome, AssessmentOutcome::Uncertain);
        assert_eq!(assessments[0].confidence, 0.0);
    }
}
