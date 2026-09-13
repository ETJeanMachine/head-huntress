use chrono::DateTime;
use chrono::Utc;
use uuid::Uuid;

pub struct RawJob {
    pub id: Uuid,
    pub source: String,
    pub url: String,
    pub fetched_at: DateTime<Utc>,
    pub content: String,
}

pub struct Job {
    pub id: Uuid, // this should just be a TID or smth internally
    pub source: String,
}
