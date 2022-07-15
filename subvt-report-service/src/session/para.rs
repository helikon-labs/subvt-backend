use crate::{ResultResponse, ServiceState, CONFIG};
use actix_web::{get, web, HttpResponse};
use serde::Deserialize;
use subvt_types::report::SessionParasVoteReport;
use subvt_types::{err::ServiceError, substrate::Epoch};

async fn validate_params(
    maybe_start_session_index: Option<i64>,
    maybe_end_session_index: Option<i64>,
    current_session: Epoch,
    min_para_vote_session_index: u64,
) -> Result<(u64, u64), HttpResponse> {
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
    // check if era range is valid
    if end_session_index - start_session_index + 1 > CONFIG.report.max_session_index_range as u64 {
        return Err(HttpResponse::BadRequest().json(ServiceError::from(&format!(
            "Bad Request: This report cannot span more than {} sessions.",
            CONFIG.report.max_session_index_range,
        ))));
    }
    Ok((start_session_index, end_session_index))
}

#[derive(Deserialize)]
pub(crate) struct SessionParasReportQueryParameters {
    #[serde(rename(deserialize = "start_session_index"))]
    maybe_start_session_index: Option<i64>,
    #[serde(rename(deserialize = "end_session_index"))]
    maybe_end_session_index: Option<i64>,
}

#[get("/report/session/paras")]
pub(crate) async fn session_paras_vote_summaries_service(
    query: web::Query<SessionParasReportQueryParameters>,
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
    let (start_session_index, end_session_index) = match validate_params(
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
        result.push(SessionParasVoteReport {
            session: data
                .postgres
                .get_epoch_by_index(session_index)
                .await?
                .unwrap(),
            paras: data
                .postgres
                .get_session_para_vote_summaries(session_index)
                .await?,
        });
    }
    Ok(HttpResponse::Ok().json(result))
}
