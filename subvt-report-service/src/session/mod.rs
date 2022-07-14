use super::{ResultResponse, ServiceState};
use actix_web::{get, web, HttpResponse};
use serde::Deserialize;
use std::sync::Arc;
use subvt_persistence::postgres::network::PostgreSQLNetworkStorage;
use subvt_types::report::{
    ParaVoteType, ParaVotesSummary, SessionValidatorParaVoteReport, SessionValidatorReport,
};
use subvt_types::substrate::Epoch;
use subvt_types::{crypto::AccountId, err::ServiceError};

async fn validate(
    ss58_address: &str,
    maybe_start_session_index: Option<i64>,
    maybe_end_session_index: Option<i64>,
    current_session: Epoch,
    min_para_vote_session_index: u64,
) -> Result<(AccountId, u64, u64), HttpResponse> {
    // check valid address
    let account_id = match AccountId::from_ss58_check(ss58_address) {
        Ok(account_id) => account_id,
        Err(_) => {
            return Err(HttpResponse::BadRequest().json(ServiceError::from(
                format!("Bad Request: Invalid address {}.", ss58_address).as_ref(),
            )))
        }
    };
    // start session index
    let start_session_index = match maybe_start_session_index {
        Some(start_session_index) => start_session_index as u64,
        None => current_session.index,
    };
    if start_session_index > current_session.index {
        return Err(HttpResponse::BadRequest().json(ServiceError::from(&format!(
            "Bad Request: start_session_index ({}) cannot be greater than the current session index ({}).",
            start_session_index, current_session.index,
        ))));
    }
    // end session index
    let end_session_index = match maybe_end_session_index {
        Some(end_session_index) => {
            if maybe_start_session_index.is_none() {
                return Err(HttpResponse::BadRequest().json(ServiceError::from(
                    "Bad Request: Please also add the start_session_index query parameter.",
                )));
            }
            if (end_session_index as u64) > current_session.index {
                current_session.index
            } else {
                end_session_index as u64
            }
        }
        None => start_session_index,
    };
    if start_session_index > end_session_index {
        return Err(HttpResponse::BadRequest().json(ServiceError::from(&format!(
            "Bad Request: start_session_index ({}) cannot be greater than end_session_index ({}).",
            start_session_index, end_session_index,
        ))));
    }
    if end_session_index > current_session.index {
        return Err(HttpResponse::BadRequest().json(ServiceError::from(&format!(
            "Bad Request: end_session_index ({}) cannot be greater than the current session index ({}).",
            end_session_index, current_session.index,
        ))));
    }
    // check if start session is supported [votes]
    if start_session_index < min_para_vote_session_index {
        return Err(HttpResponse::BadRequest().json(ServiceError::from(&format!(
            "Bad Request: start_session_index ({}) cannot be less than the earliest supported session index ({}).",
            start_session_index,
            min_para_vote_session_index,
        ))));
    }
    Ok((account_id, start_session_index, end_session_index))
}

async fn get_session_validator_report(
    postgres: &Arc<PostgreSQLNetworkStorage>,
    session_index: u64,
    validator_account_id: &AccountId,
) -> anyhow::Result<Option<SessionValidatorReport>> {
    let session = match postgres.get_epoch_by_index(session_index).await? {
        Some(session) => session,
        None => return Ok(None),
    };
    let mut report = SessionValidatorReport {
        session,
        is_active: false,
        validator_index: None,
        blocks_authored: None,
        para_validator_group_index: None,
        para_validator_index: None,
        para_votes_summary: None,
    };
    match postgres
        .get_era_validator_by_session_index(validator_account_id, session_index)
        .await?
    {
        Some(era_validator) => {
            report.is_active = era_validator.is_active;
            report.validator_index = era_validator.active_validator_index;
        }
        None => return Ok(Some(report)),
    }
    report.blocks_authored = Some(
        postgres
            .get_blocks_by_validator_in_session(session_index, validator_account_id)
            .await?,
    );
    // get heartbeat event
    // get offline offences
    match postgres
        .get_session_para_validator(session_index, validator_account_id)
        .await?
    {
        Some(session_para_validator) => {
            report.para_validator_group_index =
                Some(session_para_validator.para_validator_group_index);
            report.para_validator_index = Some(session_para_validator.para_validator_index);
        }
        None => return Ok(Some(report)),
    }
    // get votes (summary)
    let mut para_votes_summary = ParaVotesSummary::default();
    let votes = postgres
        .get_session_para_validator_votes(session_index, report.para_validator_index.unwrap())
        .await?;
    for vote in votes {
        match vote.vote {
            ParaVoteType::EXPLICIT => para_votes_summary.explicit += 1,
            ParaVoteType::IMPLICIT => para_votes_summary.implicit += 1,
            ParaVoteType::MISSED => para_votes_summary.missed += 1,
        }
    }
    report.para_votes_summary = Some(para_votes_summary);
    Ok(Some(report))
}

#[derive(Deserialize)]
pub(crate) struct SessionValidatorReportPathParameters {
    ss58_address: String,
}

#[derive(Deserialize)]
pub(crate) struct SessionValidatorReportQueryParameters {
    #[serde(rename(deserialize = "start_session_index"))]
    maybe_start_session_index: Option<i64>,
    #[serde(rename(deserialize = "end_session_index"))]
    maybe_end_session_index: Option<i64>,
}

#[get("/report/session/validator/{ss58_address}")]
pub(crate) async fn session_validator_report_service(
    path: web::Path<SessionValidatorReportPathParameters>,
    query: web::Query<SessionValidatorReportQueryParameters>,
    data: web::Data<ServiceState>,
) -> ResultResponse {
    let current_session = match data.postgres.get_current_epoch().await? {
        Some(session) => session,
        None => {
            return Ok(HttpResponse::InternalServerError().json(ServiceError::from(
                "Internal Error: Cannot get current session.",
            )))
        }
    };
    let min_para_vote_session_index = data.postgres.get_min_para_vote_session_index().await?;
    let (account_id, start_session_index, end_session_index) = match validate(
        &path.into_inner().ss58_address,
        query.maybe_start_session_index,
        query.maybe_end_session_index,
        current_session,
        min_para_vote_session_index,
    )
    .await
    {
        Ok(tuple) => tuple,
        Err(response) => return Ok(response),
    };

    let mut result = vec![];
    for session_index in start_session_index..=end_session_index {
        if let Some(report) =
            get_session_validator_report(&data.postgres, session_index, &account_id).await?
        {
            result.push(report);
        }
    }
    Ok(HttpResponse::Ok().json(result))
}

async fn get_session_validator_para_vote_report(
    postgres: &Arc<PostgreSQLNetworkStorage>,
    session_index: u64,
    validator_account_id: &AccountId,
) -> anyhow::Result<Option<SessionValidatorParaVoteReport>> {
    let session = match postgres.get_epoch_by_index(session_index).await? {
        Some(session) => session,
        None => return Ok(None),
    };
    let mut report = SessionValidatorParaVoteReport {
        session,
        para_validator_group_index: None,
        para_validator_index: None,
        para_votes_summary: None,
        para_votes: None,
    };
    match postgres
        .get_session_para_validator(session_index, validator_account_id)
        .await?
    {
        Some(session_para_validator) => {
            report.para_validator_group_index =
                Some(session_para_validator.para_validator_group_index);
            report.para_validator_index = Some(session_para_validator.para_validator_index);
        }
        None => return Ok(Some(report)),
    }
    let para_votes = postgres
        .get_session_para_validator_votes(session_index, report.para_validator_index.unwrap())
        .await?;
    let mut para_votes_summary = ParaVotesSummary::default();
    for para_vote in &para_votes {
        match para_vote.vote {
            ParaVoteType::EXPLICIT => para_votes_summary.explicit += 1,
            ParaVoteType::IMPLICIT => para_votes_summary.implicit += 1,
            ParaVoteType::MISSED => para_votes_summary.missed += 1,
        }
    }
    report.para_votes_summary = Some(para_votes_summary);
    report.para_votes = Some(para_votes);
    Ok(Some(report))
}

#[get("/report/session/validator/{ss58_address}/para_vote")]
pub(crate) async fn session_validator_para_vote_service(
    path: web::Path<SessionValidatorReportPathParameters>,
    query: web::Query<SessionValidatorReportQueryParameters>,
    data: web::Data<ServiceState>,
) -> ResultResponse {
    let current_session = match data.postgres.get_current_epoch().await? {
        Some(session) => session,
        None => {
            return Ok(HttpResponse::InternalServerError().json(ServiceError::from(
                "Internal Error: Cannot get current session.",
            )))
        }
    };
    let min_para_vote_session_index = data.postgres.get_min_para_vote_session_index().await?;
    let (account_id, start_session_index, end_session_index) = match validate(
        &path.into_inner().ss58_address,
        query.maybe_start_session_index,
        query.maybe_end_session_index,
        current_session,
        min_para_vote_session_index,
    )
    .await
    {
        Ok(tuple) => tuple,
        Err(response) => return Ok(response),
    };
    // validation passed
    let mut result = vec![];
    for session_index in start_session_index..=end_session_index {
        if let Some(report) =
            get_session_validator_para_vote_report(&data.postgres, session_index, &account_id)
                .await?
        {
            result.push(report);
        }
    }
    Ok(HttpResponse::Ok().json(result))
}
