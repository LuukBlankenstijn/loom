use chrono::{DateTime, Utc};
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct Contest {
    pub id: String,
    pub name: String,
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: DateTime<Utc>,
}
