use crate::crypto::AccountId;
use serde::{Deserialize, Serialize};

pub const PUBLIC_KEY_HEX_LENGTH: usize = 64;

#[derive(Clone, Debug, Default, Serialize)]
pub struct Network {
    pub id: u32,
    pub hash: String,
    pub name: String,
    pub live_network_status_service_url: Option<String>,
    pub report_service_url: Option<String>,
    pub validator_details_service_url: Option<String>,
    pub validator_list_service_url: Option<String>,
}

fn default_id() -> u32 {
    0
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct User {
    #[serde(default = "default_id")]
    pub id: u32,
    pub public_key_hex: String,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct NotificationChannel {
    pub code: String,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct NotificationType {
    pub code: String,
    pub param_types: Vec<NotificationParamType>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct NotificationParamType {
    pub id: u32,
    pub notification_type_code: String,
    pub order: u8,
    pub code: String,
    #[serde(rename(serialize = "type", deserialize = "type"))]
    pub type_: String,
    pub min: Option<String>,
    pub max: Option<String>,
    pub is_optional: bool,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct UserNotificationChannel {
    #[serde(default = "default_id")]
    pub id: u32,
    #[serde(default = "default_id")]
    pub user_id: u32,
    pub channel_code: String,
    pub target: String,
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

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct UserNotificationRuleParameter {
    #[serde(default = "default_id")]
    pub user_notification_rule_id: u32,
    pub parameter_type_id: u32,
    pub order: u8,
    pub value: String,
}

impl UserNotificationRuleParameter {
    pub fn validate(&self, parameter_type: &NotificationParamType) -> (bool, Option<String>) {
        match parameter_type.type_.as_ref() {
            "string" => {
                if let Some(min) = &parameter_type.min {
                    if self.value.len() < min.parse::<usize>().unwrap() {
                        return (
                            false,
                            Some(format!("String length cannot be less than {}.", min)),
                        );
                    }
                }
                if let Some(max) = &parameter_type.max {
                    if self.value.len() > max.parse::<usize>().unwrap() {
                        return (
                            false,
                            Some(format!("String length cannot be more than {}.", max)),
                        );
                    }
                }
            }
            "integer" => {
                if let Some(min) = &parameter_type.min {
                    if let Ok(value) = self.value.parse::<i64>() {
                        if value < min.parse::<i64>().unwrap() {
                            return (false, Some(format!("Cannot be less than {}.", min)));
                        }
                    } else {
                        return (false, Some("Invalid integer value.".to_string()));
                    }
                }
                if let Some(max) = &parameter_type.max {
                    if let Ok(value) = self.value.parse::<i64>() {
                        if value > max.parse::<i64>().unwrap() {
                            return (false, Some(format!("Cannot be more than {}.", max)));
                        }
                    } else {
                        return (false, Some("Invalid integer value.".to_string()));
                    }
                }
            }
            "balance" => {
                if let Some(min) = &parameter_type.min {
                    if let Ok(value) = self.value.parse::<u128>() {
                        if value < min.parse::<u128>().unwrap() {
                            return (false, Some(format!("Cannot be less than {}.", min)));
                        }
                    } else {
                        return (false, Some("Invalid balance value.".to_string()));
                    }
                }
                if let Some(max) = &parameter_type.max {
                    if let Ok(value) = self.value.parse::<u128>() {
                        if value > max.parse::<u128>().unwrap() {
                            return (false, Some(format!("Cannot be more than {}.", max)));
                        }
                    } else {
                        return (false, Some("Invalid balance value.".to_string()));
                    }
                }
            }
            "float" => {
                if let Some(min) = &parameter_type.min {
                    if let Ok(value) = self.value.parse::<f64>() {
                        if value < min.parse::<f64>().unwrap() {
                            return (false, Some(format!("Cannot be less than {}.", min)));
                        }
                    } else {
                        return (false, Some("Invalid float value.".to_string()));
                    }
                }
                if let Some(max) = &parameter_type.max {
                    if let Ok(value) = self.value.parse::<f64>() {
                        if value > max.parse::<f64>().unwrap() {
                            return (false, Some(format!("Cannot be more than {}.", max)));
                        }
                    } else {
                        return (false, Some("Invalid float value.".to_string()));
                    }
                }
            }
            "boolean" => {
                if self.value.parse::<bool>().is_err() {
                    return (false, Some("Invalid boolean value.".to_string()));
                }
            }
            _ => unreachable!("Unexpected parameter type: {}", parameter_type.type_),
        }
        (true, None)
    }
}

#[derive(Clone, Debug, Default, Serialize)]
pub struct UserNotificationRule {
    pub id: u32,
    pub notification_type: NotificationType,
    pub name: Option<String>,
    pub network: Option<Network>,
    pub is_for_all_validators: bool,
    pub validators: Vec<UserValidator>,
    pub notification_channels: Vec<UserNotificationChannel>,
    pub parameters: Vec<UserNotificationRuleParameter>,
    pub notes: Option<String>,
}
