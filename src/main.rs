mod options {
    tonic::include_proto!("options");
}

mod query;
mod server;

use std::path::PathBuf;
use std::net::SocketAddr;

use tokio::net::TcpListener;
use tokio::signal;
use tokio_stream::wrappers::TcpListenerStream;
use tracing::{info, error};
use tracing_subscriber::EnvFilter;

use server::OptionsQueryServiceImpl;

const DEFAULT_LISTEN_ADDR: &str = "0.0.0.0:50051";
const DEFAULT_DATA_DIR: &str = "data/options_data";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive("info".parse()?))
        .init();

    let listen_addr: SocketAddr = std::env::var("LISTEN_ADDR")
        .unwrap_or_else(|_| DEFAULT_LISTEN_ADDR.to_string())
        .parse()?;

    let data_dir = std::env::var("DATA_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from(DEFAULT_DATA_DIR));

    if !data_dir.exists() {
        error!(path = %data_dir.display(), "Data directory does not exist");
        std::process::exit(1);
    }

    info!(addr = %listen_addr, "Server starting");
    info!(path = %data_dir.display(), "Data directory");

    let service_impl = OptionsQueryServiceImpl::new(data_dir);
    let server = service_impl.into_server();

    let listener = TcpListener::bind(listen_addr).await?;
    let incoming = TcpListenerStream::new(listener);

    let svc = tonic::transport::Server::builder()
        .add_service(server)
        .serve_with_incoming_shutdown(incoming, shutdown_signal());

    info!("Server listening on {}", listen_addr);

    svc.await?;

    info!("Shutting down...");

    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    info!("Received shutdown signal");
}
