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

    // Global DNS entries
    #[clap(long, short, default_value = "foobar:127.0.0.1", env = "EVNDNS_GLOBAL_ENTRIES")]
    pub entries: Vec<String>,
}

#[derive(Debug, Clone, Parser)]
pub struct ClientOptions {
    // DNS entry list
    #[clap(long, short, default_value = "foobar:127.0.0.1", env = "EVNDNS_ENTRIES")]
    pub entries: Vec<String>,
}