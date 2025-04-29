use crate::groundlink::{proto::skycanvas::groundlink::groundlink_service_server::GroundlinkServiceServer, services::svc_ardulink_connect::SvcArdulinkConnect};
use log::info;

use std::net::SocketAddr;
use tokio::task;
use tonic::transport::Server;
use tonic_web::GrpcWebLayer;

pub async fn start_groundlink_server(addr: SocketAddr) -> Result<task::JoinHandle<()>, anyhow::Error> {
    let service = SvcArdulinkConnect {};
    let service = GroundlinkServiceServer::new(service);

    
    info!("Groundlink server starting on {}", addr);
    
    let handle = task::spawn(async move {
        if let Err(e) = Server::builder()
            .accept_http1(true)
            .add_service(service)
            .serve(addr)
            .await
        {
            log::error!("Groundlink server error: {}", e);
        }
        info!("Groundlink server stopped");
    });
    
    Ok(handle)
}

pub async fn start_default_groundlink_server() -> Result<task::JoinHandle<()>, anyhow::Error> {
    let addr = "0.0.0.0:5050".parse()?;
    start_groundlink_server(addr).await
}
