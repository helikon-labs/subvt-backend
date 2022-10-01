//! Types used in the application logic of SubVT.
use crate::crypto::AccountId;
use serde::{Deserialize, Serialize};

pub mod app_event;
pub mod db;
pub mod event;
pub mod extrinsic;
pub mod notification;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Block {
    pub hash: String,
    pub number: u64,
    pub timestamp: Option<u64>,
    pub author_account_id: Option<AccountId>,
    pub era_index: u64,
    pub epoch_index: u64,
    pub is_finalized: bool,
    pub metadata_version: u16,
    pub runtime_version: u16,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct Network {
    pub id: u32,
    pub hash: String,
    pub chain: String,
    pub display: String,
    pub ss58_prefix: u32,
    pub token_ticker: String,
    pub token_decimal_count: u8,
    pub network_status_service_host: Option<String>,
    pub network_status_service_port: Option<u16>,
    pub report_service_host: Option<String>,
    pub report_service_port: Option<u16>,
    pub validator_details_service_host: Option<String>,
    pub validator_details_service_port: Option<u16>,
    pub active_validator_list_service_host: Option<String>,
    pub active_validator_list_service_port: Option<u16>,
    pub inactive_validator_list_service_host: Option<String>,
    pub inactive_validator_list_service_port: Option<u16>,
}

fn default_id() -> u32 {
    0
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct User {
    #[serde(default = "default_id")]
    pub id: u32,
    pub public_key_hex: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct UserValidator {
    #[serde(default = "default_id")]
    pub id: u32,
    #[serde(default = "default_id")]
    pub user_id: u32,
    pub network_id: u32,
    pub validator_account_id: AccountId,
}
