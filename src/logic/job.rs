use chrono::DateTime;
use chrono::Utc;

pub struct JobId {}

pub struct RawJob {
    pub source: String,
    pub url: String,
    pub fetched_at: DateTime<Utc>,
    pub content: String,
}

pub struct Job {
    pub id: String, // this should just be a TID or smth internally
    pub source: String,
}
