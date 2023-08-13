use anyhow::Result;
use clap::Parser;
use handler::Handler;
use options::{Options, ClientOptions};
use std::time::Duration;
use tokio::net::{TcpListener, UdpSocket};
use trust_dns_server::ServerFuture;

mod options;
mod handler;
mod resolver;

/// Timeout for TCP connections
const TCP_TIMEOUT: Duration = Duration::from_secs(5);

#[derive(Parser, Debug)]
#[clap(about = "DNS Application")]
enum Command {
    #[clap(about = "Starts in server mode.")]
    Server(Options),
    #[clap(about = "Starts in client mode.")]
    Client(ClientOptions),
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    
    match Command::parse() {
        Command::Server(options) => start_server(options).await,
        Command::Client(client_options) => start_client(client_options).await,
    }
}

async fn start_client(client_options: ClientOptions) -> Result<()> {
    // Your client logic here
    println!("Starting as a client...");

    Ok(())
}

async fn start_server(options: Options) -> Result<()> {
    let handler = Handler::from_options(&options);

    // create DNS server
    let mut server = ServerFuture::new(handler);
    
    // register UDP listeners
    for udp in &options.udp {
        server.register_socket(UdpSocket::bind(udp).await?);
    }

    // register TCP listeners
    for tcp in &options.tcp {
        server.register_listener(TcpListener::bind(&tcp).await?, TCP_TIMEOUT);
    }

    // run DNS server
    server.block_until_done().await?;

    Ok(())
}
