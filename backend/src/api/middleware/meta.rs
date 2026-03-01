use axum::{
    body::Body,
    extract::ConnectInfo,
    http::{Request, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};
use std::net::{IpAddr, SocketAddr};
use tonic::Status;

#[derive(Clone, Debug)]
pub struct ClientMeta {
    pub ip: IpAddr,
    pub host: String,
}

pub trait RequestExt {
    fn client_meta(&self) -> Result<&ClientMeta, Status>;
}

impl<T> RequestExt for tonic::Request<T> {
    fn client_meta(&self) -> Result<&ClientMeta, Status> {
        self.extensions()
            .get::<ClientMeta>()
            .ok_or_else(|| Status::internal("Client metadata not found in request extensions"))
    }
}

pub async fn client_meta_interceptor(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    mut req: Request<Body>,
    next: Next,
) -> Response {
    let real_ip = req
        .headers()
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.split(',').next())
        .and_then(|v| v.trim().parse::<IpAddr>().ok())
        .unwrap_or_else(|| addr.ip());

    let host = match req.uri().authority().map(|a| a.to_string()).or_else(|| {
        req.headers()
            .get("host")
            .and_then(|v| v.to_str().ok())
            .map(|v| v.to_string())
    }) {
        Some(h) => h,
        None => {
            return (StatusCode::BAD_REQUEST, "Missing host or authority header").into_response();
        }
    };

    req.extensions_mut()
        .insert(ClientMeta { ip: real_ip, host });

    next.run(req).await
}
