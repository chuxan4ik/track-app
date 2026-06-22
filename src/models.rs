use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, Clone)]
pub struct Delivery {
    pub id: i32,
    pub tracking_number: String,
    pub description: String,
    pub sender: String,
    pub recipient: String,
    pub current_status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct NewDelivery {
    pub tracking_number: String,
    pub description: String,
    pub sender: String,
    pub recipient: String,
}

#[derive(Debug, Deserialize)]
pub struct StatusUpdate {
    pub status: String,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct StatusHistory {
    pub id: i32,
    pub delivery_id: i32,
    pub status: String,
    pub changed_at: DateTime<Utc>,
}