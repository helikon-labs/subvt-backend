//! Application REST interface. Contains services such as user registration, network list,
//! notification channels, user validator registration, user notification rules persistence
//! and deletion, etc.
use actix_web::web::Data;
use actix_web::{delete, get, post, web, App, HttpResponse, HttpServer};
use async_trait::async_trait;
use lazy_static::lazy_static;
use log::debug;
use serde::Deserialize;
use std::collections::HashSet;
use std::sync::Arc;
use subvt_config::Config;
use subvt_persistence::postgres::app::PostgreSQLAppStorage;
use subvt_service_common::{err::InternalServerError, Service};
use subvt_types::app::{
    NotificationPeriodType, User, UserNotificationChannel, UserNotificationRuleParameter,
    UserValidator, PUBLIC_KEY_HEX_LENGTH,
};
use subvt_types::err::ServiceError;

lazy_static! {
    static ref CONFIG: Config = Config::default();
}

type ResultResponse = Result<HttpResponse, InternalServerError>;

#[derive(Clone)]
pub struct ServiceState {
    pub postgres: Arc<PostgreSQLAppStorage>,
}

async fn check_user_exists_by_id(
    state: &web::Data<ServiceState>,
    user_id: u32,
) -> anyhow::Result<Option<HttpResponse>> {
    if !state.postgres.user_exists_by_id(user_id).await? {
        return Ok(Some(
            HttpResponse::NotFound().json(ServiceError::from("User not found.".to_string())),
        ));
    }
    Ok(None)
}

/// `GET`s the list of networks supported by SubVT.
#[get("/service/network")]
pub async fn get_networks(state: web::Data<ServiceState>) -> ResultResponse {
    Ok(HttpResponse::Ok().json(state.postgres.get_networks().await?))
}

/// `GET`s the list of supported notification channels, such as email, push notification, SMS, etc.
#[get("/service/notification/channel")]
async fn get_notification_channels(state: web::Data<ServiceState>) -> ResultResponse {
    Ok(HttpResponse::Ok().json(state.postgres.get_notification_channels().await?))
}

/// `GET`s the list of notification types and their parameters supported by SubVT.
#[get("/service/notification/type")]
async fn get_notification_types(state: web::Data<ServiceState>) -> ResultResponse {
    Ok(HttpResponse::Ok().json(state.postgres.get_notification_types().await?))
}

/// Validates and creates a new user.
#[post("/service/user")]
async fn create_user(state: web::Data<ServiceState>, mut user: web::Json<User>) -> ResultResponse {
    let public_key_hex = user.public_key_hex.trim_start_matches("0x").to_uppercase();
    // validate public key hex length
    if public_key_hex.len() != PUBLIC_KEY_HEX_LENGTH {
        return Ok(HttpResponse::BadRequest().json(ServiceError::from(format!(
            "Public key should be {} characters long hexadecimal string.",
            PUBLIC_KEY_HEX_LENGTH
        ))));
    }
    // validate hex format
    if hex::decode(&public_key_hex).is_err() {
        return Ok(HttpResponse::BadRequest().json(ServiceError::from(
            "Public key should be valid hexadecimal string.".to_string(),
        )));
    }
    user.public_key_hex = format!("0x{}", public_key_hex);
    // check duplicate public key
    if state
        .postgres
        .user_exists_with_public_key(&user.public_key_hex)
        .await?
    {
        return Ok(HttpResponse::Conflict().json(ServiceError::from(
            "A user exists with the given public key.".to_string(),
        )));
    }
    user.id = state.postgres.save_user(&user).await?;
    Ok(HttpResponse::Created().json(user))
}

#[derive(Deserialize)]
struct UserIdPathParameter {
    pub user_id: u32,
}

/// `GET`s the list of notification channels that the user has created for herself so far.
#[get("/service/user/{user_id}/notification/channel")]
async fn get_user_notification_channels(
    path_params: web::Path<UserIdPathParameter>,
    state: web::Data<ServiceState>,
) -> ResultResponse {
    if let Some(error_response) = check_user_exists_by_id(&state, path_params.user_id).await? {
        return Ok(error_response);
    }
    Ok(HttpResponse::Ok().json(
        state
            .postgres
            .get_user_notification_channels(path_params.user_id)
            .await?,
    ))
}

/// Creates a new notification channel for the user.
#[post("/service/user/{user_id}/notification/channel")]
async fn add_user_notification_channel(
    path_params: web::Path<UserIdPathParameter>,
    mut input: web::Json<UserNotificationChannel>,
    state: web::Data<ServiceState>,
) -> ResultResponse {
    input.user_id = path_params.user_id as u32;
    if let Some(error_response) = check_user_exists_by_id(&state, input.user_id).await? {
        return Ok(error_response);
    }
    // check notification channel exists
    if !state
        .postgres
        .notification_channel_exists(&input.channel_code)
        .await?
    {
        return Ok(HttpResponse::NotFound().json(ServiceError::from(
            "Notification channel not found.".to_string(),
        )));
    }
    if state
        .postgres
        .user_notification_channel_target_exists(&input)
        .await?
    {
        return Ok(HttpResponse::Conflict().json(ServiceError::from(
            "This target exists for the user.".to_string(),
        )));
    }
    // validate input
    if input.target.is_empty() {
        return Ok(HttpResponse::BadRequest().json(ServiceError::from(
            "Invalid notification target.".to_string(),
        )));
    }
    input.id = state
        .postgres
        .save_user_notification_channel(&input)
        .await?;
    Ok(HttpResponse::Created().json(input))
}

#[derive(Deserialize)]
struct UserNotificationChannelIdPathParameter {
    pub user_id: u32,
    pub channel_id: u32,
}

/// `DELETE`s the notification channel from the user's list of notification channels.
/// A soft delete, but the user will no longer receive notifications on this channel.
#[delete("/service/user/{user_id}/notification/channel/{channel_id}")]
async fn delete_user_notification_channel(
    path_params: web::Path<UserNotificationChannelIdPathParameter>,
    state: web::Data<ServiceState>,
) -> ResultResponse {
    let channel_exists = state
        .postgres
        .user_notification_channel_exists(path_params.user_id, path_params.channel_id)
        .await?;
    if !channel_exists {
        return Ok(HttpResponse::NotFound().json(ServiceError::from(
            "User notification channel not found.".to_string(),
        )));
    }
    match state
        .postgres
        .delete_user_notification_channel(path_params.channel_id)
        .await?
    {
        true => Ok(HttpResponse::NoContent().finish()),
        false => Ok(HttpResponse::InternalServerError().json(ServiceError::from(
            "There was an error deleting the notification channel.".to_string(),
        ))),
    }
}

/// `GET`s the list of all validators registered to the user.
#[get("/service/user/{user_id}/validator")]
async fn get_user_validators(
    path_params: web::Path<UserIdPathParameter>,
    state: web::Data<ServiceState>,
) -> ResultResponse {
    if let Some(error_response) = check_user_exists_by_id(&state, path_params.user_id).await? {
        return Ok(error_response);
    }
    Ok(HttpResponse::Ok().json(
        state
            .postgres
            .get_user_validators(path_params.user_id)
            .await?,
    ))
}

/// Adds a new validator to the user's list of validators.
#[post("/service/user/{user_id}/validator")]
async fn add_user_validator(
    path_params: web::Path<UserIdPathParameter>,
    mut input: web::Json<UserValidator>,
    state: web::Data<ServiceState>,
) -> ResultResponse {
    input.user_id = path_params.user_id;
    if let Some(error_response) = check_user_exists_by_id(&state, input.user_id).await? {
        return Ok(error_response);
    }
    // check network exists
    if !state
        .postgres
        .network_exists_by_id(input.network_id)
        .await?
    {
        return Ok(
            HttpResponse::NotFound().json(ServiceError::from("Network not found.".to_string()))
        );
    }
    // check user validator exists
    if state.postgres.user_validator_exists(&input).await? {
        return Ok(
            HttpResponse::Conflict().json(ServiceError::from("User validator exists.".to_string()))
        );
    }
    input.id = state.postgres.save_user_validator(&input).await?;
    Ok(HttpResponse::Created().json(input))
}

#[derive(Deserialize)]
struct UserValidatorIdPathParameter {
    pub user_id: u32,
    pub user_validator_id: u32,
}

/// `DELETE`s a validator from the user's list of validators.
/// A soft delete, i.e. only marks the validator as deleted.
#[delete("/service/user/{user_id}/validator/{user_validator_id}")]
async fn delete_user_validator(
    path_params: web::Path<UserValidatorIdPathParameter>,
    state: web::Data<ServiceState>,
) -> ResultResponse {
    // check validator exists
    if !state
        .postgres
        .user_validator_exists_by_id(path_params.user_id, path_params.user_validator_id)
        .await?
    {
        return Ok(HttpResponse::NotFound()
            .json(ServiceError::from("User validator not found.".to_string())));
    }
    match state
        .postgres
        .delete_user_validator(path_params.user_validator_id)
        .await?
    {
        true => Ok(HttpResponse::NoContent().finish()),
        false => Ok(HttpResponse::InternalServerError().json(ServiceError::from(
            "There was an error deleting the user's validator.".to_string(),
        ))),
    }
}

#[derive(Deserialize)]
struct CreateUserNotificationRuleRequest {
    pub notification_type_code: String,
    pub name: Option<String>,
    pub network_id: Option<u32>,
    pub is_for_all_validators: bool,
    pub user_validator_ids: HashSet<u32>,
    pub period_type: NotificationPeriodType,
    pub period: u16,
    pub user_notification_channel_ids: HashSet<u32>,
    pub parameters: Vec<UserNotificationRuleParameter>,
    pub notes: Option<String>,
}

/// `GET`s the list of the user's non-deleted notification rules.
#[get("/service/user/{user_id}/notification/rule")]
async fn get_user_notification_rules(
    path_params: web::Path<UserIdPathParameter>,
    state: web::Data<ServiceState>,
) -> ResultResponse {
    if let Some(error_response) = check_user_exists_by_id(&state, path_params.user_id).await? {
        return Ok(error_response);
    }
    Ok(HttpResponse::Ok().json(
        state
            .postgres
            .get_user_notification_rules(path_params.user_id)
            .await?,
    ))
}

/// Creates a new notification rule for the user. The new rule starts getting evaluated for possible
/// notifications as soon as it gets created.
#[post("/service/user/{user_id}/notification/rule")]
async fn create_user_notification_rule(
    path_params: web::Path<UserIdPathParameter>,
    mut input: web::Json<CreateUserNotificationRuleRequest>,
    state: web::Data<ServiceState>,
) -> ResultResponse {
    if let Some(error_response) = check_user_exists_by_id(&state, path_params.user_id).await? {
        return Ok(error_response);
    }
    // check notification type exists
    if !state
        .postgres
        .notification_type_exists_by_code(&input.notification_type_code)
        .await?
    {
        return Ok(HttpResponse::NotFound().json(ServiceError::from(
            "Notification type not found.".to_string(),
        )));
    }
    // check network exists
    if let Some(network_id) = input.network_id {
        if !state.postgres.network_exists_by_id(network_id).await? {
            return Ok(
                HttpResponse::NotFound().json(ServiceError::from("Network not found.".to_string()))
            );
        }
    }
    // check validators
    if input.is_for_all_validators {
        input.user_validator_ids.clear();
    } else if input.user_validator_ids.is_empty() {
        return Ok(HttpResponse::BadRequest().json(ServiceError::from(
            "At least 1 user validator should be selected.".to_string(),
        )));
    }
    for user_validator_id in &input.user_validator_ids {
        if !state
            .postgres
            .user_validator_exists_by_id(path_params.user_id, *user_validator_id)
            .await?
        {
            return Ok(HttpResponse::NotFound()
                .json(ServiceError::from("User validator not found.".to_string())));
        }
    }
    // check if there is at least one notification channel
    if input.user_notification_channel_ids.is_empty() {
        return Ok(HttpResponse::BadRequest().json(ServiceError::from(
            "There should be at least 1 notification channel selected.".to_string(),
        )));
    }
    // check user notification channel ids
    for user_notification_channel_id in &input.user_notification_channel_ids {
        if !state
            .postgres
            .user_notification_channel_exists(path_params.user_id, *user_notification_channel_id)
            .await?
        {
            return Ok(HttpResponse::NotFound().json(ServiceError::from(
                "User notification channel not found.".to_string(),
            )));
        }
    }
    let notification_parameter_types = state
        .postgres
        .get_notification_parameter_types(&input.notification_type_code)
        .await?;
    let notification_parameter_type_ids: Vec<u32> = notification_parameter_types
        .iter()
        .map(|parameter_type| parameter_type.id)
        .collect();
    let irrelevant_parameter_type_ids: Vec<u32> = input
        .parameters
        .iter()
        .map(|parameter| parameter.parameter_type_id)
        .filter(|id| !notification_parameter_type_ids.contains(id))
        .collect();
    if !irrelevant_parameter_type_ids.is_empty() {
        return Ok(HttpResponse::NotFound().json(ServiceError::from(format!(
            "Posted parameter(s) with id(s) {:?} not found for notification type '{}'.",
            irrelevant_parameter_type_ids, input.notification_type_code
        ))));
    }
    let posted_parameter_type_ids: Vec<u32> = input
        .parameters
        .iter()
        .map(|parameter| parameter.parameter_type_id)
        .collect();
    // check if all non-optional parameters are sent
    let missing_non_optional_parameter_type_ids: Vec<u32> = notification_parameter_types
        .iter()
        .filter(|parameter_type| {
            !parameter_type.is_optional && !posted_parameter_type_ids.contains(&parameter_type.id)
        })
        .map(|parameter_type| parameter_type.id)
        .collect();
    if !missing_non_optional_parameter_type_ids.is_empty() {
        return Ok(HttpResponse::NotFound().json(ServiceError::from(format!(
            "Missing non-optional parameter type ids: {:?}",
            missing_non_optional_parameter_type_ids
        ))));
    }
    // validate parameters
    for parameter in &input.parameters {
        let parameter_type = notification_parameter_types
            .iter()
            .find(|parameter_type| parameter_type.id == parameter.parameter_type_id)
            .unwrap();
        if let (false, Some(validation_error_message)) = parameter.validate(parameter_type) {
            return Ok(HttpResponse::BadRequest().json(ServiceError::from(format!(
                "Invalid '{}': {}",
                parameter_type.code, validation_error_message
            ))));
        }
    }
    let rule_id = state
        .postgres
        .save_user_notification_rule(
            path_params.user_id,
            &input.notification_type_code,
            (input.name.as_deref(), input.notes.as_deref()),
            (input.network_id, input.is_for_all_validators),
            (&input.period_type, input.period),
            (
                &input.user_validator_ids,
                &input.user_notification_channel_ids,
                &input.parameters,
            ),
        )
        .await?;
    // get rule
    Ok(HttpResponse::Created().json(
        state
            .postgres
            .get_user_notification_rule_by_id(rule_id)
            .await?,
    ))
}

#[derive(Deserialize)]
struct UserNotificationRuleIdPathParameter {
    pub user_id: u32,
    pub user_notification_rule_id: u32,
}

/// `DELETE` a rule from the list of the user's notification rules.
/// A soft delete, the rule will not be able to generate new notifications as soon as
/// it gets deleted.
#[delete("/service/user/{user_id}/notification/rule/{user_notification_rule_id}")]
async fn delete_user_notification_rule(
    path_params: web::Path<UserNotificationRuleIdPathParameter>,
    state: web::Data<ServiceState>,
) -> ResultResponse {
    // check rule exists
    if !state
        .postgres
        .user_notification_rule_exists_by_id(
            path_params.user_id,
            path_params.user_notification_rule_id,
        )
        .await?
    {
        return Ok(HttpResponse::NotFound().json(ServiceError::from(
            "User notification rule not found.".to_string(),
        )));
    }
    match state
        .postgres
        .delete_user_notification_rule(path_params.user_notification_rule_id)
        .await?
    {
        true => Ok(HttpResponse::NoContent().finish()),
        false => Ok(HttpResponse::InternalServerError().json(ServiceError::from(
            "There was an error deleting the user notification rule.".to_string(),
        ))),
    }
}

async fn on_server_ready() {
    debug!("HTTP service started.");
}

#[derive(Default)]
pub struct AppService;

/// Service implmentation.
#[async_trait(?Send)]
impl Service for AppService {
    async fn run(&'static self) -> anyhow::Result<()> {
        // persistence instance
        let postgres =
            Arc::new(PostgreSQLAppStorage::new(&CONFIG, CONFIG.get_app_postgres_url()).await?);
        debug!("Starting HTTP service.");
        let server = HttpServer::new(move || {
            App::new()
                .app_data(Data::new(ServiceState {
                    postgres: postgres.clone(),
                }))
                .app_data(web::JsonConfig::default().error_handler(|err, _req| {
                    actix_web::error::InternalError::from_response(
                        "",
                        HttpResponse::BadRequest().json(ServiceError::from(format!("{}", err))),
                    )
                    .into()
                }))
                .service(get_networks)
                .service(get_notification_channels)
                .service(get_notification_types)
                .service(create_user)
                .service(add_user_notification_channel)
                .service(get_user_notification_channels)
                .service(delete_user_notification_channel)
                .service(get_user_validators)
                .service(add_user_validator)
                .service(delete_user_validator)
                .service(create_user_notification_rule)
                .service(get_user_notification_rules)
                .service(delete_user_notification_rule)
        })
        .workers(10)
        .disable_signals()
        .bind(format!(
            "{}:{}",
            CONFIG.http.host, CONFIG.http.app_service_port,
        ))?
        .run();
        let (server_result, _) = tokio::join!(server, on_server_ready());
        Ok(server_result?)
    }
}
