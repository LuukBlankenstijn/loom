pub fn check_auth(
    req: tonic::Request<()>,
    expected_secret: Option<String>,
) -> Result<tonic::Request<()>, tonic::Status> {
    if let Some(secret) = expected_secret {
        let auth_header = req.metadata().get("authorization");
        let expected = format!("Bearer {}", secret);

        match auth_header {
            Some(t) if t == expected.as_str() => Ok(req),
            Some(_) => {
                tracing::warn!("unauthenticated: invalid token");
                Err(tonic::Status::unauthenticated("Invalid secret"))
            }
            None => {
                tracing::warn!("unauthenticated: no token provided");
                Err(tonic::Status::unauthenticated("Missing secret"))
            }
        }
    } else {
        Ok(req)
    }
}
