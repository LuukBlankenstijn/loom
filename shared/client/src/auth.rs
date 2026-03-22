use tonic::{
    metadata::{Ascii, MetadataValue},
    service::Interceptor,
};

#[derive(Debug, Clone)]
pub struct AuthInterceptor {
    token: Option<MetadataValue<Ascii>>,
}

impl AuthInterceptor {
    pub fn new() -> Self {
        Self { token: None }
    }

    pub fn with_auth(&mut self, token: String) -> Result<Self, String> {
        let token_value: MetadataValue<_> = token
            .parse()
            .map_err(|_| "Invalid auth token, could not convert to metadata value".to_string())?;
        let mut new = self.clone();
        new.token = Some(token_value);
        Ok(new)
    }
}

impl Interceptor for AuthInterceptor {
    fn call(&mut self, request: tonic::Request<()>) -> Result<tonic::Request<()>, tonic::Status> {
        let mut req = request;
        if let Some(token) = self.token.clone() {
            req.metadata_mut().insert("authorization", token);
        };

        Ok(req)
    }
}
