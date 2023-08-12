use clap::Parser;
use std::net::SocketAddr;

#[derive(Debug, Clone, Parser)]
pub struct Options {
    // UDP socket to listen on
    #[clap(long, short, default_value = "0.0.0.0:1053", env = "EVNDNS_UPD")]
    pub udp: Vec<SocketAddr>,

    // TCP socket to listen on
    #[clap(long, short, env = "EVNDNS_TCP")]
    pub tcp: Vec<SocketAddr>,

    // Domain name
    #[clap(long, short, default_value = "evndns.local", env = "EVNDNS_DOMAIN")]
    pub domain: String,

    // Upstream DNS server
    #[clap(long, short, default_value = "8.8.8.8", env = "EVNDNS_UPSTREAM")]
    pub upstream: SocketAddr,
}