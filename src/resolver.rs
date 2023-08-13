use trust_dns_resolver::{TokioAsyncResolver, error::ResolveError}; // Added ResolveError to the imports
use trust_dns_server::client::rr::{Name, RData, Record};
use std::net::IpAddr;
use thiserror::Error;
use once_cell::sync::Lazy;
use std::sync::Arc;

#[derive(Error, Debug, Clone)]
pub enum ResolverError {
    #[error("IO error: {0:?}")]
    Io(#[source] Arc<std::io::Error>),  // Wrap the error inside an Arc
    #[error("Resolver error: {0:?}")]
    Dns(#[source] Arc<ResolveError>),   // New variant for DNS resolver errors
}

impl From<std::io::Error> for ResolverError {
    fn from(err: std::io::Error) -> Self {
        ResolverError::Io(Arc::new(err))  // Wrap the error inside an Arc when converting
    }
}

impl From<ResolveError> for ResolverError {  // Conversion from ResolveError to ResolverError
    fn from(err: ResolveError) -> Self {
        ResolverError::Dns(Arc::new(err))
    }
}

// Singleton for TokioAsyncResolver
pub static RESOLVER: Lazy<Result<TokioAsyncResolver, ResolverError>> = Lazy::new(|| {
    TokioAsyncResolver::tokio_from_system_conf().map_err(ResolverError::from)
});

pub async fn resolve(name: Name) -> Result<Vec<Record>, ResolverError> {
    // Use the singleton resolver
    let resolver = &*RESOLVER.as_ref().map_err(|e| e.clone())?;
    
    // Clone the name before passing it to lookup_ip
    let lookup = resolver.lookup_ip(name.clone()).await.map_err(ResolverError::from)?; 

    let records: Vec<Record> = lookup
        .iter()
        .map(|ip| Record::from_rdata(name.clone(), 60, match ip {
            IpAddr::V4(ipv4) => RData::A(ipv4),
            IpAddr::V6(ipv6) => RData::AAAA(ipv6),
        }))
        .collect();

    Ok(records)
}
