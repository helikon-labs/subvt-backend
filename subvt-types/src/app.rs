use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct AppServiceError {
    pub description: String,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Network {
    pub id: i32,
    pub hash: String,
    pub name: String,
    pub app_service_url: Option<String>,
    pub live_network_status_service_url: Option<String>,
    pub report_service_url: Option<String>,
    pub validator_details_service_url: Option<String>,
    pub validator_list_service_url: Option<String>,
}
