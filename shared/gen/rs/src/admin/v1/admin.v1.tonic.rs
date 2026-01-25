// @generated
/// Generated client implementations.
pub mod admin_service_client {
    #![allow(
        unused_variables,
        dead_code,
        missing_docs,
        clippy::wildcard_imports,
        clippy::let_unit_value,
    )]
    use tonic::codegen::*;
    use tonic::codegen::http::Uri;
    ///
    #[derive(Debug, Clone)]
    pub struct AdminServiceClient<T> {
        inner: tonic::client::Grpc<T>,
    }
    impl AdminServiceClient<tonic::transport::Channel> {
        /// Attempt to create a new client by connecting to a given endpoint.
        pub async fn connect<D>(dst: D) -> Result<Self, tonic::transport::Error>
        where
            D: TryInto<tonic::transport::Endpoint>,
            D::Error: Into<StdError>,
        {
            let conn = tonic::transport::Endpoint::new(dst)?.connect().await?;
            Ok(Self::new(conn))
        }
    }
    impl<T> AdminServiceClient<T>
    where
        T: tonic::client::GrpcService<tonic::body::Body>,
        T::Error: Into<StdError>,
        T::ResponseBody: Body<Data = Bytes> + std::marker::Send + 'static,
        <T::ResponseBody as Body>::Error: Into<StdError> + std::marker::Send,
    {
        pub fn new(inner: T) -> Self {
            let inner = tonic::client::Grpc::new(inner);
            Self { inner }
        }
        pub fn with_origin(inner: T, origin: Uri) -> Self {
            let inner = tonic::client::Grpc::with_origin(inner, origin);
            Self { inner }
        }
        pub fn with_interceptor<F>(
            inner: T,
            interceptor: F,
        ) -> AdminServiceClient<InterceptedService<T, F>>
        where
            F: tonic::service::Interceptor,
            T::ResponseBody: Default,
            T: tonic::codegen::Service<
                http::Request<tonic::body::Body>,
                Response = http::Response<
                    <T as tonic::client::GrpcService<tonic::body::Body>>::ResponseBody,
                >,
            >,
            <T as tonic::codegen::Service<
                http::Request<tonic::body::Body>,
            >>::Error: Into<StdError> + std::marker::Send + std::marker::Sync,
        {
            AdminServiceClient::new(InterceptedService::new(inner, interceptor))
        }
        /// Compress requests with the given encoding.
        ///
        /// This requires the server to support it otherwise it might respond with an
        /// error.
        #[must_use]
        pub fn send_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.inner = self.inner.send_compressed(encoding);
            self
        }
        /// Enable decompressing responses.
        #[must_use]
        pub fn accept_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.inner = self.inner.accept_compressed(encoding);
            self
        }
        /// Limits the maximum size of a decoded message.
        ///
        /// Default: `4MB`
        #[must_use]
        pub fn max_decoding_message_size(mut self, limit: usize) -> Self {
            self.inner = self.inner.max_decoding_message_size(limit);
            self
        }
        /// Limits the maximum size of an encoded message.
        ///
        /// Default: `usize::MAX`
        #[must_use]
        pub fn max_encoding_message_size(mut self, limit: usize) -> Self {
            self.inner = self.inner.max_encoding_message_size(limit);
            self
        }
        /** Gets the current or next active contest
*/
        pub async fn get_next_contest(
            &mut self,
            request: impl tonic::IntoRequest<()>,
        ) -> std::result::Result<tonic::Response<super::Contest>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::unknown(
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic_prost::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/admin.v1.AdminService/GetNextContest",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("admin.v1.AdminService", "GetNextContest"));
            self.inner.unary(req, path, codec).await
        }
        /** Gets all teams in the current or next active contest
*/
        pub async fn get_active_teams(
            &mut self,
            request: impl tonic::IntoRequest<()>,
        ) -> std::result::Result<tonic::Response<super::TeamsResponse>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::unknown(
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic_prost::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/admin.v1.AdminService/GetActiveTeams",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("admin.v1.AdminService", "GetActiveTeams"));
            self.inner.unary(req, path, codec).await
        }
        /** Gets all registered stations
*/
        pub async fn get_stations(
            &mut self,
            request: impl tonic::IntoRequest<()>,
        ) -> std::result::Result<
            tonic::Response<super::StationsResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::unknown(
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic_prost::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/admin.v1.AdminService/GetStations",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("admin.v1.AdminService", "GetStations"));
            self.inner.unary(req, path, codec).await
        }
        /** Sets the ip for some team. Only allows ips that are not used by a different team yet
*/
        pub async fn set_ip(
            &mut self,
            request: impl tonic::IntoRequest<super::SetIpRequest>,
        ) -> std::result::Result<tonic::Response<()>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::unknown(
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic_prost::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/admin.v1.AdminService/SetIp",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("admin.v1.AdminService", "SetIp"));
            self.inner.unary(req, path, codec).await
        }
        /** Sets the wallpaper for some contest
*/
        pub async fn set_wallpaper(
            &mut self,
            request: impl tonic::IntoRequest<super::UploadImageRequest>,
        ) -> std::result::Result<tonic::Response<()>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::unknown(
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic_prost::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/admin.v1.AdminService/SetWallpaper",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("admin.v1.AdminService", "SetWallpaper"));
            self.inner.unary(req, path, codec).await
        }
    }
}
