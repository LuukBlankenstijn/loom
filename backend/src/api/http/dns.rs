use std::{net::Ipv4Addr, str::FromStr};

use axum::{
    extract::{Query, State},
    http::{HeaderMap, Response, StatusCode},
    response::IntoResponse,
};
use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use bytes::Bytes;
use reqwest::header;
use serde::Deserialize;
use simple_dns::{
    CLASS, Packet, PacketFlag, QTYPE, RCODE, ResourceRecord, TYPE,
    rdata::{A, OPT, RData},
};
use utoipa::IntoParams;

use crate::{api::http::HttpHandlerState, error::AppError};

const DNS_MESSAGE: &str = "application/dns-message";

#[utoipa::path(
    post,
    path = "/dns-query",
    request_body(
        description = "DNS query in wire format (RFC 1035), at most 65535 bytes",
        content(("application/dns-message")),
    ),
    tag = "dns",
    responses(
        (status = 200, description = "DNS response in wire format. Carries NXDOMAIN for an unknown team, REFUSED outside team.loom, and SERVFAIL when the team lookup fails", content_type = "application/dns-message"),
        (status = 400, description = "Body is not a parseable DNS packet"),
        (status = 413, description = "Body exceeds 65535 bytes"),
        (status = 415, description = "Content-Type is not application/dns-message"),
    )
)]
pub async fn dns_post(
    State(state): State<HttpHandlerState>,
    headers: HeaderMap,
    body: Bytes,
) -> Response<axum::body::Body> {
    if !is_dns_message(&headers) {
        return StatusCode::UNSUPPORTED_MEDIA_TYPE.into_response();
    }
    respond(body, state).await
}

fn is_dns_message(headers: &HeaderMap) -> bool {
    headers
        .get(header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.split(';').next())
        .is_some_and(|media_type| media_type.trim().eq_ignore_ascii_case(DNS_MESSAGE))
}

#[derive(Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct DnsQuery {
    /// base64 encoded DNS query (RFC 1035)
    dns: String,
}

#[utoipa::path(
    get,
    path = "/dns-query",
    tag = "dns",
    params(DnsQuery),
    responses(
        (status = 200, description = "DNS response in wire format. Carries NXDOMAIN for an unknown team, REFUSED outside team.loom, and SERVFAIL when the team lookup fails", content_type = "application/dns-message"),
        (status = 400, description = "The dns parameter is missing, is not base64url, or does not decode to a parseable DNS packet"),
        (status = 413, description = "Decoded query exceeds 65535 bytes"),
    )
)]
pub async fn dns_get(
    State(state): State<HttpHandlerState>,
    Query(data): Query<DnsQuery>,
) -> Response<axum::body::Body> {
    let Ok(bytes) = URL_SAFE_NO_PAD.decode(data.dns.trim_end_matches('=')) else {
        return StatusCode::BAD_REQUEST.into_response();
    };
    respond(bytes.into(), state).await
}

async fn respond(bytes: Bytes, state: HttpHandlerState) -> Response<axum::body::Body> {
    if bytes.len() > 65535 {
        return StatusCode::PAYLOAD_TOO_LARGE.into_response();
    }
    let Ok(query) = Packet::parse(&bytes) else {
        return StatusCode::BAD_REQUEST.into_response();
    };

    let mut reply = Packet::new_reply(query.id());
    reply.set_flags(PacketFlag::AUTHORITATIVE_ANSWER);
    if query.has_flags(PacketFlag::RECURSION_DESIRED) {
        reply.set_flags(PacketFlag::RECURSION_DESIRED);
    }
    reply.questions = query.questions.clone();
    if query.opt().is_some() {
        *reply.opt_mut() = Some(OPT {
            opt_codes: vec![],
            udp_packet_size: 4096,
            version: 0,
        });
    }

    let mut max_age = 60;

    let Some(q) = query.questions.first() else {
        *reply.rcode_mut() = RCODE::FormatError;
        return finish(reply, max_age);
    };

    let name = q.qname.to_string().to_ascii_lowercase();
    let Some(team) = name
        .strip_suffix(".team.loom")
        .filter(|t| !t.is_empty() && !t.contains('.'))
    else {
        *reply.rcode_mut() = RCODE::Refused;
        return finish(reply, max_age);
    };

    match lookup(team, state).await {
        Ok(None) => *reply.rcode_mut() = RCODE::NameError,
        Ok(Some(ip)) if q.qtype == QTYPE::TYPE(TYPE::A) => {
            reply.answers.push(ResourceRecord::new(
                q.qname.clone(),
                CLASS::IN,
                300,
                RData::A(A {
                    address: u32::from(ip),
                }),
            ));
            max_age = 300;
        }
        Ok(Some(_)) => {}
        Err(error) => {
            tracing::error!(%error, team, "dns lookup failed");
            *reply.rcode_mut() = RCODE::ServerFailure;
            max_age = 0;
        }
    }

    finish(reply, max_age)
}

fn finish(reply: Packet, max_age: u32) -> Response<axum::body::Body> {
    let Ok(bytes) = reply.build_bytes_vec() else {
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    };
    (
        StatusCode::OK,
        [
            (header::CONTENT_TYPE, DNS_MESSAGE.to_string()),
            (header::CACHE_CONTROL, format!("max-age={max_age}")),
        ],
        bytes,
    )
        .into_response()
}

async fn lookup(team_id: &str, state: HttpHandlerState) -> Result<Option<Ipv4Addr>, AppError> {
    let Some(team) = state.team_repo.get(team_id).await? else {
        return Ok(None);
    };
    match team.ip.as_ref().map(|s| Ipv4Addr::from_str(s)) {
        None => Ok(None),
        Some(Ok(ip)) => Ok(Some(ip)),
        Some(Err(_)) => {
            tracing::error!(ip = &team.ip, "failed to parse ip");
            Ok(None)
        }
    }
}
