use std::net::UdpSocket;

use clap::Parser;
use loom_rpc::stations::v1::RegisterRequest;
use loom_rpc::stations::v1::station_service_client::StationServiceClient;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Server to connect to
    #[arg(short, long)]
    server: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    println!("server: {}", args.server);

    let mut client = StationServiceClient::connect(args.server.clone()).await?;

    let request = tonic::Request::new(RegisterRequest {
        ip: get_local_ip(&args.server),
    });

    let mut stream = client.subscribe(request).await?.into_inner();

    while let Some(response) = stream.message().await? {
        println!("Received {:?}", response);
    }

    Ok(())
}

fn get_local_ip(target: &str) -> String {
    let host = target
        .trim_start_matches("http://")
        .trim_start_matches("https://")
        .split('/')
        .next()
        .unwrap_or(target);

    let socket = UdpSocket::bind("0.0.0.0:0").expect("Could not bind local socket");

    match socket.connect(format!("{}:80", host)) {
        Ok(_) => socket
            .local_addr()
            .map(|addr| addr.ip().to_string())
            .unwrap_or_else(|_| "127.0.0.1".to_string()),
        Err(_) => "127.0.0.1".to_string(),
    }
}
