mod domain;
mod rpc_client;

use anyhow::Result;

pub use domain::{AdminService, Contest, Station, Team};

pub async fn make_admin_service(endpoint: String) -> Result<impl AdminService> {
    let conn = tonic::transport::Endpoint::new(endpoint)?.connect().await?;
    Ok(rpc_client::RpcProvider::new(conn))
}
