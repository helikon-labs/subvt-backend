//! Application REST interface. Contains services such as user registration, network list,
//! notification channels, user validator registration, user notification rules persistence
//! and deletion, etc.
#![warn(clippy::disallowed_types)]
use crate::auth::{data::AuthenticatedUser, service::AuthServiceFactory};
use actix_web::{delete, get, post, web, App, HttpRequest, HttpResponse, HttpServer};
use async_trait::async_trait;
use lazy_static::lazy_static;
use rustc_hash::FxHashSet as HashSet;
use serde::Deserialize;
use std::sync::Arc;
use subvt_config::Config;
use subvt_persistence::postgres::app::PostgreSQLAppStorage;
use subvt_service_common::{err::InternalServerError, Service};
use subvt_types::app::{
    notification::{
        NotificationPeriodType, UserNotificationChannel, UserNotificationRuleParameter,
    },
    User, UserValidator,
};
use subvt_types::err::ServiceError;

mod auth;
pub(crate) mod metrics;

lazy_static! {
    static ref CONFIG: Config = Config::default();
}

type ResultResponse = Result<HttpResponse, InternalServerError>;

#[derive(Clone)]
pub struct ServiceState {
    pub postgres: Arc<PostgreSQLAppStorage>,
}

/// `GET`s the list of networks supported by SubVT.
#[get("/network")]
pub async fn get_networks(state: web::Data<ServiceState>) -> ResultResponse {
    Ok(HttpResponse::Ok().json(state.postgres.get_networks().await?))
}

/// `GET`s the list of supported notification channels, such as email, push notification, SMS, etc.
#[get("/notification/channel")]
async fn get_notification_channels(state: web::Data<ServiceState>) -> ResultResponse {
    Ok(HttpResponse::Ok().json(state.postgres.get_notification_channels().await?))
}

/// `GET`s the list of notification types and their parameters supported by SubVT.
#[get("/notification/type")]
async fn get_notification_types(state: web::Data<ServiceState>) -> ResultResponse {
    Ok(HttpResponse::Ok().json(state.postgres.get_notification_types().await?))
}

/// Validates and creates a new user.
#[post("/secure/user")]
async fn create_user(state: web::Data<ServiceState>, request: HttpRequest) -> ResultResponse {
    // rate limit per IP address
    let maybe_registration_ip = request
        .connection_info()
        .realip_remote_addr()
        .map(str::to_string);
    if let Some(registration_ip) = &maybe_registration_ip {
        let count = state
            .postgres
            .get_user_registration_count_from_ip(registration_ip)
            .await?;
        if count > (CONFIG.app_service.user_registration_per_ip_limit as u64) {
            return Ok(HttpResponse::TooManyRequests().json(ServiceError::from(
                "Too many create user requests from the same IP address.",
            )));
        }
    }
    let public_key_hex = request
        .headers()
        .get("SubVT-Public-Key")
        .unwrap()
        .to_str()
        .unwrap();
    let public_key_hex = if !public_key_hex.starts_with("0x") {
        format!("0x{}", public_key_hex.to_uppercase())
    } else {
        public_key_hex.to_uppercase()
    };
    // check duplicate public key
    if state
        .postgres
        .user_exists_by_public_key(&public_key_hex)
        .await?
    {
        return Ok(HttpResponse::Conflict().json(ServiceError::from(
            "A user exists with the given public key.",
        )));
    }
    let mut user = User {
        id: 0,
        public_key_hex: Some(public_key_hex),
    };
    user.id = state
        .postgres
        .save_user(&user, maybe_registration_ip.as_deref())
        .await?;
    Ok(HttpResponse::Created().json(user))
}

/// `GET`s the list of notification channels that the user has created for herself so far.
#[get("/secure/user/notification/channel")]
async fn get_user_notification_channels(
    state: web::Data<ServiceState>,
    auth: AuthenticatedUser,
) -> ResultResponse {
    Ok(HttpResponse::Ok().json(
        state
            .postgres
            .get_user_notification_channels(auth.id)
            .await?,
    ))
}

/// Creates a new notification channel for the user.
#[post("/secure/user/notification/channel")]
async fn add_user_notification_channel(
    mut input: web::Json<UserNotificationChannel>,
    state: web::Data<ServiceState>,
    auth: AuthenticatedUser,
) -> ResultResponse {
    input.user_id = auth.id;
    // check notification channel type exists
    if !state
        .postgres
        .notification_channel_exists(&input.channel)
        .await?
    {
        return Ok(
            HttpResponse::NotFound().json(ServiceError::from("Notification channel not found."))
        );
    }
    // validate input
    if input.target.is_empty() {
        return Ok(
            HttpResponse::BadRequest().json(ServiceError::from("Invalid notification target."))
        );
    }
    // if channel exists, just return it
    let user_notification_channels = state
        .postgres
        .get_user_notification_channels(auth.id)
        .await?;
    for channel in user_notification_channels.iter() {
        if channel.channel == input.channel && channel.target == input.target {
            log::info!(
                "{} channel with target {} exists for user {}. Not recreating.",
                channel.channel,
                channel.target,
                auth.id
            );
            return Ok(HttpResponse::Ok().json(channel));
        }
    }
    // delete existing channels with the same code, possibly for other users
    let deleted_channel_count = state
        .postgres
        .delete_existing_notification_channels_with_code(
            input.channel.to_string().as_str(),
            input.target.to_string().as_str(),
        )
        .await?;
    log::debug!(
        "Deleted {} existing {} channels with the same code while adding a new notification channel.",
        deleted_channel_count,
        input.channel.to_string().as_str(),
    );
    input.id = state
        .postgres
        .save_user_notification_channel(&input)
        .await?;
    Ok(HttpResponse::Created().json(input))
}

#[derive(Deserialize)]
struct IdPathParameter {
    pub id: u32,
}

/// `DELETE`s the notification channel from the user's list of notification channels.
/// A soft delete, but the user will no longer receive notifications on this channel.
#[delete("/secure/user/notification/channel/{id}")]
async fn delete_user_notification_channel(
    path_params: web::Path<IdPathParameter>,
    state: web::Data<ServiceState>,
    auth: AuthenticatedUser,
) -> ResultResponse {
    let channel_exists = state
        .postgres
        .user_notification_channel_exists(auth.id, path_params.id)
        .await?;
    if !channel_exists {
        return Ok(HttpResponse::NotFound()
            .json(ServiceError::from("User notification channel not found.")));
    }
    match state
        .postgres
        .delete_user_notification_channel(path_params.id)
        .await?
    {
        true => Ok(HttpResponse::NoContent().finish()),
        false => Ok(HttpResponse::InternalServerError().json(ServiceError::from(
            "There was an error deleting the notification channel.",
        ))),
    }
}

/// `GET`s the list of all validators registered to the user.
#[get("/secure/user/validator")]
async fn get_user_validators(
    state: web::Data<ServiceState>,
    auth: AuthenticatedUser,
) -> ResultResponse {
    Ok(HttpResponse::Ok().json(state.postgres.get_user_validators(auth.id).await?))
}

/// Adds a new validator to the user's list of validators.
#[post("/secure/user/validator")]
async fn add_user_validator(
    mut input: web::Json<UserValidator>,
    state: web::Data<ServiceState>,
    auth: AuthenticatedUser,
) -> ResultResponse {
    input.user_id = auth.id;
    // check network exists
    if !state
        .postgres
        .network_exists_by_id(input.network_id)
        .await?
    {
        return Ok(HttpResponse::NotFound().json(ServiceError::from("Network not found.")));
    }
    // check user validator exists
    if state.postgres.user_validator_exists(&input).await? {
        return Ok(HttpResponse::Conflict().json(ServiceError::from("User validator exists.")));
    }
    input.id = state.postgres.save_user_validator(&input).await?;
    Ok(HttpResponse::Created().json(input))
}

/// `DELETE`s a validator from the user's list of validators.
/// A soft delete, i.e. only marks the validator as deleted.
#[delete("/secure/user/validator/{id}")]
async fn delete_user_validator(
    path_params: web::Path<IdPathParameter>,
    state: web::Data<ServiceState>,
    auth: AuthenticatedUser,
) -> ResultResponse {
    // check validator exists
    if !state
        .postgres
        .user_validator_exists_by_id(auth.id, path_params.id)
        .await?
    {
        return Ok(HttpResponse::NotFound().json(ServiceError::from("User validator not found.")));
    }
    match state.postgres.delete_user_validator(path_params.id).await? {
        true => Ok(HttpResponse::NoContent().finish()),
        false => Ok(HttpResponse::InternalServerError().json(ServiceError::from(
            "There was an error deleting the user's validator.",
        ))),
    }
}

#[derive(Deserialize)]
struct CreateDefaultUserNotificationRulesRequest {
    pub user_notification_channel_id: u32,
}

#[post("/secure/user/notification/rule/default")]
async fn create_default_user_notification_rules(
    input: web::Json<CreateDefaultUserNotificationRulesRequest>,
    state: web::Data<ServiceState>,
    auth: AuthenticatedUser,
) -> ResultResponse {
    let mut channel_id_set = HashSet::default();
    channel_id_set.insert(input.user_notification_channel_id);
    if state.postgres.user_has_created_rules(auth.id).await? {
        let rules = state.postgres.get_user_notification_rules(auth.id).await?;
        log::info!(
            "User {} has already created rules. Add notification channel to {} rules if not added already.",
            auth.id,
            rules.len()
        );
        for rule in rules.iter() {
            state
                .postgres
                .add_user_notification_channel_to_rule(
                    input.user_notification_channel_id as i32,
                    rule.id as i32,
                )
                .await?;
        }
        return Ok(HttpResponse::NoContent().finish());
    }
    log::info!("Create default notification rules for user {}.", auth.id);
    for rule in subvt_types::app::notification::rules::DEFAULT_RULES.iter() {
        state
            .postgres
            .save_user_notification_rule(
                auth.id,
                &rule.0.to_string(),
                (None, None),
                (None, true),
                (&rule.1, rule.2),
                (&HashSet::default(), &channel_id_set, &[]),
            )
            .await?;
    }
    Ok(HttpResponse::NoContent().finish())
}

/// `GET`s the list of the user's non-deleted notification rules.
#[get("/secure/user/notification/rule")]
async fn get_user_notification_rules(
    state: web::Data<ServiceState>,
    auth: AuthenticatedUser,
) -> ResultResponse {
    Ok(HttpResponse::Ok().json(state.postgres.get_user_notification_rules(auth.id).await?))
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

/// Creates a new notification rule for the user. The new rule starts getting evaluated for possible
/// notifications as soon as it gets created.
#[post("/secure/user/notification/rule")]
async fn create_user_notification_rule(
    mut input: web::Json<CreateUserNotificationRuleRequest>,
    state: web::Data<ServiceState>,
    auth: AuthenticatedUser,
) -> ResultResponse {
    // check notification type exists
    if let Some(notification_type) = state
        .postgres
        .get_notification_type_by_code(&input.notification_type_code)
        .await?
    {
        if !notification_type.is_enabled {
            return Ok(HttpResponse::BadRequest()
                .json(ServiceError::from("Notification type is not enabled.")));
        }
    } else {
        return Ok(
            HttpResponse::NotFound().json(ServiceError::from("Notification type not found."))
        );
    }
    // check network exists
    if let Some(network_id) = input.network_id {
        if !state.postgres.network_exists_by_id(network_id).await? {
            return Ok(HttpResponse::NotFound().json(ServiceError::from("Network not found.")));
        }
    }
    // check validators
    if input.is_for_all_validators {
        input.user_validator_ids.clear();
    } else if input.user_validator_ids.is_empty() {
        return Ok(HttpResponse::BadRequest().json(ServiceError::from(
            "At least 1 user validator should be selected.",
        )));
    }
    for user_validator_id in &input.user_validator_ids {
        if !state
            .postgres
            .user_validator_exists_by_id(auth.id, *user_validator_id)
            .await?
        {
            return Ok(
                HttpResponse::NotFound().json(ServiceError::from("User validator not found."))
            );
        }
    }
    // check if there is at least one notification channel
    if input.user_notification_channel_ids.is_empty() {
        return Ok(HttpResponse::BadRequest().json(ServiceError::from(
            "There should be at least 1 notification channel selected.",
        )));
    }
    // check user notification channel ids
    for user_notification_channel_id in &input.user_notification_channel_ids {
        if !state
            .postgres
            .user_notification_channel_exists(auth.id, *user_notification_channel_id)
            .await?
        {
            return Ok(HttpResponse::NotFound()
                .json(ServiceError::from("User notification channel not found.")));
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
        return Ok(HttpResponse::NotFound().json(ServiceError::from(
            format!(
                "Posted parameter(s) with id(s) {:?} not found for notification type '{}'.",
                irrelevant_parameter_type_ids, input.notification_type_code
            )
            .as_ref(),
        )));
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
        return Ok(HttpResponse::BadRequest().json(ServiceError::from(
            format!(
                "Missing non-optional parameter type ids: {missing_non_optional_parameter_type_ids:?}",
            )
            .as_ref(),
        )));
    }
    // validate parameters
    for parameter in &input.parameters {
        let parameter_type = notification_parameter_types
            .iter()
            .find(|parameter_type| parameter_type.id == parameter.parameter_type_id)
            .unwrap();
        if let (false, Some(validation_error_message)) = parameter.validate(parameter_type) {
            return Ok(HttpResponse::BadRequest().json(ServiceError::from(
                format!(
                    "Invalid '{}': {validation_error_message}",
                    parameter_type.code,
                )
                .as_ref(),
            )));
        }
    }
    let rule_id = state
        .postgres
        .save_user_notification_rule(
            auth.id,
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

/// `DELETE` a rule from the list of the user's notification rules.
/// A soft delete, the rule will not be able to generate new notifications as soon as
/// it gets deleted.
#[delete("/secure/user/notification/rule/{id}")]
async fn delete_user_notification_rule(
    path_params: web::Path<IdPathParameter>,
    state: web::Data<ServiceState>,
    auth: AuthenticatedUser,
) -> ResultResponse {
    // check rule exists
    if !state
        .postgres
        .user_notification_rule_exists_by_id(auth.id, path_params.id)
        .await?
    {
        return Ok(
            HttpResponse::NotFound().json(ServiceError::from("User notification rule not found."))
        );
    }
    match state
        .postgres
        .delete_user_notification_rule(path_params.id)
        .await?
    {
        true => Ok(HttpResponse::NoContent().finish()),
        false => Ok(HttpResponse::InternalServerError().json(ServiceError::from(
            "There was an error deleting the user notification rule.",
        ))),
    }
}

async fn on_server_ready() {
    log::debug!("HTTP service started.");
}

#[derive(Default)]
pub struct AppService;

/// Service implementation.
#[async_trait(?Send)]
impl Service for AppService {
    fn get_metrics_server_addr() -> (&'static str, u16) {
        (
            CONFIG.metrics.host.as_str(),
            CONFIG.metrics.app_service_port,
        )
    }

    async fn run(&'static self) -> anyhow::Result<()> {
        // persistence instance
        let postgres =
            Arc::new(PostgreSQLAppStorage::new(&CONFIG, CONFIG.get_app_postgres_url()).await?);
        log::debug!("Starting HTTP service.");
        let server = HttpServer::new(move || {
            App::new()
                .app_data(web::Data::new(ServiceState {
                    postgres: postgres.clone(),
                }))
                .app_data(web::JsonConfig::default().error_handler(|err, _req| {
                    actix_web::error::InternalError::from_response(
                        "",
                        HttpResponse::BadRequest()
                            .json(ServiceError::from(format!("{err}").as_ref())),
                    )
                    .into()
                }))
                .wrap(AuthServiceFactory {})
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
                .service(create_default_user_notification_rules)
        })
        .workers(10)
        .disable_signals()
        .bind(format!(
            "{}:{}",
            CONFIG.http.service_host, CONFIG.http.app_service_port,
        ))?
        .run();
        let (server_result, _) = tokio::join!(server, on_server_ready());
        Ok(server_result?)
    }
}
