use serde::{Deserialize, Serialize};

pub const PUBLIC_KEY_HEX_LENGTH: usize = 64;

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct AppServiceError {
    pub description: String,
}

impl AppServiceError {
    pub fn from(description: String) -> AppServiceError {
        AppServiceError { description }
    }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Network {
    pub id: u32,
    pub hash: String,
    pub name: String,
    pub live_network_status_service_url: Option<String>,
    pub report_service_url: Option<String>,
    pub validator_details_service_url: Option<String>,
    pub validator_list_service_url: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct User {
    pub id: u32,
    pub public_key_hex: String,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct NotificationChannel {
    pub name: String,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct NotificationType {
    pub id: u32,
    pub code: String,
}
